use std::{
    collections::HashMap,
    io, net,
    sync::{self, mpsc},
};

use crate::shared::{frame, mux, stream};

pub type Accept = Result<stream::ShareableStream, io::Error>;

// Mux multiplexes many Streams over one Framer.
pub struct Mux {
    framer: sync::Arc<sync::Mutex<frame::Framer>>,
    streams: HashMap<u32, stream::ShareableStream>,

    accept: Option<mpsc::Sender<Accept>>,
    close_once: sync::Once,
}

impl Mux {
    // NewMux creates a Mux. Call serve() in a goroutine to start reading frames.
    pub fn new(conn: net::TcpStream) -> (mux::Mux, mpsc::Receiver<Accept>) {
        let framer = frame::Framer::new(conn);
        let (accept, accept_rec) = mpsc::channel();
        let close_once = sync::Once::new();
        (
            mux::Mux {
                framer: sync::Arc::new(sync::Mutex::new(framer)),
                streams: HashMap::new(),
                accept: Some(accept),
                close_once,
            },
            accept_rec,
        )
    }

    pub fn read_frame(&mut self) -> io::Result<frame::Frame> {
        self.framer.lock().unwrap().read_frame()
    }

    // Serve reads frames forever and dispatches to the right stream.
    // Blocks until the connection is closed.
    pub fn serve(&mut self) {
        loop {
            let fr = self.framer.lock().unwrap().read_frame();
            if let Err(err) = fr {
                self.close_once.call_once(|| {
                    self.accept.as_mut().unwrap().send(Err(err)).unwrap();
                    drop(self.accept.take());
                });
                return;
            }
            let fr = fr.unwrap();
            match fr.frame_type {
                frame::FrameType::FramePing => {
                    _ = self.framer.lock().unwrap().write_frame(frame::Frame {
                        frame_type: frame::FrameType::FramePong,
                        stream_id: fr.stream_id,
                        payload: Box::default(),
                    })
                }
                frame::FrameType::FramePong => { /*keepalive reply — nothing to do*/ }
                frame::FrameType::FrameData => {
                    let s = self.get_or_create(fr.stream_id);
                    s.lock().unwrap().ingest(fr.payload);
                }
                frame::FrameType::FrameEOF => self
                    .streams
                    .get(&fr.stream_id)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .signal_eof(),
                frame::FrameType::FrameReset => self
                    .streams
                    .remove(&fr.stream_id)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .signal_eof(),
            }
        }
    }

    // Returns the next inbound stream opened by the remote.
    pub fn accept(&self, receiver: &mpsc::Receiver<Accept>) -> Accept {
        receiver.recv().unwrap()
    }

    // Creates a new outbound stream with the given ID.
    fn open_stream(&mut self, id: u32) -> stream::ShareableStream {
        let s = sync::Arc::new(sync::Mutex::new(stream::Stream::new(
            id,
            sync::Arc::clone(&self.framer),
        )));
        let s_copy = sync::Arc::clone(&s);

        self.streams.insert(id, s).unwrap();
        s_copy
    }

    fn get_or_create(&mut self, id: u32) -> stream::ShareableStream {
        if let Some(s) = self.streams.get_mut(&id) {
            return sync::Arc::clone(s);
        };
        let s = self.open_stream(id);
        let s_copy = sync::Arc::clone(&s);
        // notify Accept waiters that a new inbound stream arrived
        self.accept.as_mut().unwrap().send(Ok(s)).unwrap();
        s_copy
    }
}
