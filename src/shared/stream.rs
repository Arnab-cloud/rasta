use crate::shared::frame;
use std::cmp::min;
use std::io;
use std::sync;

// Stream is a virtual bidirectional channel inside the tunnel connection.
// It implements io.ReadWriteCloser.
pub struct Stream<'a> {
    id: u32,
    framer: &'a frame::Framer,
    buf: sync::Mutex<Box<[u8]>>,
    buf_cv: sync::Condvar,
    closed: sync::RwLock<bool>, // closedMu  sync.RWMutex
    remote_eof: bool,           // remote half-closed (no more data coming)
}

impl Stream<'_> {
    fn new(id: u32, framer: &frame::Framer) -> Stream<'_> {
        Stream {
            id,
            framer,
            buf: sync::Mutex::new(Box::default()),
            buf_cv: sync::Condvar::new(),
            closed: sync::RwLock::new(true),
            remote_eof: false,
        }
    }

    // ID returns the stream's identifier.
    fn id(&self) -> u32 {
        return self.id;
    }

    // Write sends data to the remote side.
    fn write(&self, p: Box<[u8]>) -> io::Result<usize> {
        let closed = self.closed.read().unwrap();
        let payload_len = p.len();
        if *closed {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("stream {}: write on closed stream", self.id),
            ));
        }

        match self.framer.write_frame(frame::Frame {
            frame_type: frame::FrameType::FrameData,
            stream_id: self.id,
            payload: p,
        }) {
            Ok(()) => Ok(payload_len),
            Err(err) => Err(err),
        }
    }

    // Read blocks until data is available from the remote side.
    fn read(&self, mut p: Box<[u8]>) -> io::Result<usize> {
        loop {
            if self.buf.lock().unwrap().len() != 0 {
                break;
            }

            if self.remote_eof {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }
            if *self.closed.read().unwrap() {
                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
            }

            drop(self.buf_cv.wait(self.buf.lock().unwrap()).unwrap()); // not sure what I have done
        }
        let mut buf = self.buf.lock().unwrap();
        let size_to_be_copied = min(p.len(), buf.len());
        p.copy_from_slice(&buf[..size_to_be_copied]);

        let old_box = std::mem::replace(&mut *buf, Box::default());
        let mut vec = old_box.into_vec();
        let new_vec = vec.split_off(size_to_be_copied);
        *buf = new_vec.into_boxed_slice();
        Ok(size_to_be_copied)
    }

    // CloseWrite sends EOF to the remote, signaling no more data from our side.
    fn close_write(&self) -> io::Result<()> {
        return self.framer.write_frame(frame::Frame {
            frame_type: frame::FrameType::FrameEOF,
            stream_id: self.id,
            payload: Box::default(),
        });
    }

    // Close terminates the stream locally and notifies the remote.
    fn close(&self) -> io::Result<()> {
        if *self.closed.read().unwrap() {
            return Ok(());
        }
        *self.closed.write().unwrap() = true;

        self.buf_cv.notify_all(); // wake any blocked Read
        self.framer.write_frame(frame::Frame {
            frame_type: frame::FrameType::FrameReset,
            stream_id: self.id,
            payload: Box::default(),
        })
    }

    // ingest is called by the Mux to deliver data received from the network.
    fn ingest(&mut self, data: Box<[u8]>) {
        let mut buf = self.buf.lock().unwrap();
        let mut buf_vec = buf.clone().into_vec();
        buf_vec.append(&mut data.into_vec());
        *buf = buf_vec.into_boxed_slice();
        self.buf_cv.notify_one();
    }

    // signalEOF marks the remote write side as done.
    fn signal_eof(&mut self) {
        let buf = self.buf.lock().unwrap();
        self.remote_eof = true;
        self.buf_cv.notify_all();
        drop(buf);
    }
}
