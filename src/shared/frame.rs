use std::io::{self, Read, Write};
use std::net;
use std::sync;

// FrameType identifies the purpose of a frame.
#[repr(u8)]
pub enum FrameType {
    // FrameData carries raw HTTP request/response bytes for a stream.
    FrameData = 0,
    // FrameEOF signals that a stream's write side is done (half-close).
    FrameEOF = 1,
    // FrameReset abruptly terminates a stream (e.g. upstream error).
    FrameReset = 2,
    // FramePing is a keepalive sent by the server; daemon replies with FramePong.
    FramePing = 3,
    // FramePong is the daemon's reply to a ping.
    FramePong = 4,
}

impl TryFrom<u8> for FrameType {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(FrameType::FrameData),
            1 => Ok(FrameType::FrameEOF),
            2 => Ok(FrameType::FrameReset),
            3 => Ok(FrameType::FramePing),
            4 => Ok(FrameType::FramePong),
            _ => Err(()),
        }
    }
}

const HEADER_SIZE: usize = 1 + 4 + 4; // type + streamID + length

// Frame is a single multiplexed message.
struct Frame {
    frame_type: FrameType,
    stream_id: u32,
    payload: Box<[u8]>,
}

// Framer serialises/deserialises frames on a single net.Conn.
// It is safe for concurrent writes (reads must be done by a single goroutine).
pub struct Framer {
    conn: sync::Mutex<net::TcpStream>,
}

// NewFramer wraps any ReadWriter (typically a net.Conn).
impl Framer {
    fn new(conn: net::TcpStream) -> Framer {
        return Framer {
            conn: sync::Mutex::new(conn),
        };
    }

    // WriteFrame sends a frame atomically.
    fn write_frame(&self, frame: Frame) -> io::Result<()> {
        let mut conn = self.conn.lock().unwrap();

        let mut header: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        header[0] = (frame.frame_type as u8).to_be_bytes()[0];
        header[1..5].copy_from_slice(&frame.stream_id.to_be_bytes());
        header[5..9].copy_from_slice(&frame.payload.len().to_be_bytes());

        conn.write(&header)?;

        if frame.payload.len() == 0 {
            return Ok(());
        }

        match conn.write(&frame.payload) {
            Err(err) => Err(err),
            _ => Ok(()),
        }
    }

    // ReadFrame blocks until a full frame is available.
    // Must be called by exactly one goroutine.
    fn read_frame(&self) -> io::Result<Frame> {
        let mut header: Vec<u8> = Vec::with_capacity(HEADER_SIZE);
        let mut conn = self.conn.lock().unwrap();
        conn.read_to_end(&mut header)?;

        let frame_type = match FrameType::try_from(header[0]) {
            Ok(val) => val,
            Err(_) => return Err(io::Error::from(io::ErrorKind::Other)),
        };

        let stream_id = u32::from_be_bytes(header[1..5].try_into().unwrap());
        let length = u32::from_be_bytes(header[5..9].try_into().unwrap());
        return Ok(Frame {
            frame_type,
            stream_id,
            payload: if length > 0 {
                let mut payload: Vec<u8> = Vec::with_capacity(length as usize);
                conn.read_to_end(&mut payload)?;
                payload.into_boxed_slice()
            } else {
                Box::new([])
            },
        });
    }
}
