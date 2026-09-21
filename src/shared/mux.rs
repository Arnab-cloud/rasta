use std::{io, net};

use crate::shared::frame;

// Mux multiplexes many Streams over one Framer.
pub struct Mux {
    framer: frame::Framer,
    // streams: HashMap<u32, stream::Stream<'a>>, // map[u32]*Stream

    // accept: mpsc::Receiver<stream::Stream<'a>>, // acceptErr chan error

    // close_once: sync::Once,
    // done: mpsc::Sender<()>,
}

impl Mux {
    // NewMux creates a Mux. Call Serve() in a goroutine to start reading frames.
    pub fn new(conn: net::TcpStream) -> Mux {
        let framer = frame::Framer::new(conn);
        Mux { framer }
        // 	return &Mux{
        // 		framer:    framer,
        // 		accept:    make(chan *Stream, 64),
        // 		acceptErr: make(chan error, 1),
        // 		done:      make(chan struct{}),
        // 	}
    }

    pub fn read_frame(&self) -> io::Result<frame::Frame> {
        self.framer.read_frame()
    }

    // Serve reads frames forever and dispatches to the right stream.
    // Blocks until the connection is closed.
    fn serve(&self) {
        // loop {
        // 	fr, err := self.framer.read_frame();
        // 	if err != nil {
        // 		m.closeOnce.Do(fn() {
        // 			select {
        // 			case m.acceptErr <- err:
        // 			default:
        // 			}
        // 			close(m.done)
        // 		})
        // 		return
        // 	}
    }

    // 		switch fr.frame_type {
    // 		case FramePing:
    // 			_ = m.framer.WriteFrame(Frame{frame_type: FramePong, stream_id: fr.stream_id})

    // 		case FramePong:
    // 			// keepalive reply — nothing to do

    // 		case FrameData:
    // 			s := m.getOrCreate(fr.stream_id)
    // 			s.ingest(fr.Payload)

    // 		case FrameEOF:
    // 			if s, ok := m.streams.Load(fr.stream_id); ok {
    // 				s.(*Stream).signalEOF()
    // 			}

    // 		case FrameReset:
    // 			if s, ok := m.streams.LoadAndDelete(fr.stream_id); ok {
    // 				s.(*Stream).signalEOF()
    // 			}
    // 		}
    // 	}
    // }
}

// // OpenStream creates a new outbound stream with the given ID.
// fn (m *Mux) OpenStream(id u32) *Stream {
// 	s := newStream(id, m.framer)
// 	m.streams.Store(id, s)
// 	return s
// }

// // Accept returns the next inbound stream opened by the remote.
// fn (m *Mux) Accept() (*Stream, error) {
// 	select {
// 	case s := <-m.accept:
// 		return s, nil
// 	case err := <-m.acceptErr:
// 		return nil, err
// 	case <-m.done:
// 		return nil, io.EOF
// 	}
// }

// fn (m *Mux) getOrCreate(id u32) *Stream {
// 	if v, ok := m.streams.Load(id); ok {
// 		return v.(*Stream)
// 	}
// 	s := newStream(id, m.framer)
// 	m.streams.Store(id, s)
// 	// notify Accept waiters that a new inbound stream arrived
// 	select {
// 	case m.accept <- s:
// 	default:
// 	}
// 	return s
// }

// // RemoveStream cleans up a stream from the mux table.
// fn (m *Mux) RemoveStream(id u32) {
// 	m.streams.Delete(id)
// }
