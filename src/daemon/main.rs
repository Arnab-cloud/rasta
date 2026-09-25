// // daemon connects to the tunnel server and forwards tunneled HTTP requests
// // to a local port on the host machine.
// //
// // Usage (Windows or Linux):
// //
// //	go run ./daemon --server=YOUR_SERVER:8080 --port=3000
// //
// // Flags:
// //
// //	--server   address of the tunnel server   (default: localhost:8080)
// //	--port     local port to forward to       (default: 3000)
// //	--retry    seconds between reconnects     (default: 5)
// //

// // var (
// // 	serverAddr = flag.String("server", "localhost:8080", "tunnel server address (host:port)")
// // 	localPort  = flag.Int("port", 3000, "local port to forward to")
// // 	retryDelay = flag.Int("retry", 5, "reconnect delay in seconds")
// // )

// use rasta::shared::mux;

// // // handshake must match the constant in server/main.go exactly.
// const HANDSHAKE: &str = "TUNNEL-CONNECT\n";
// const TIMEOUT: time::Duration = time::Duration::from_secs(10);

use core::net;
use std::env;

struct CmdArgs {
    server_addr: net::SocketAddr,
    local_port: u32,
    retry_delay: u32,
}

fn parse_flags() -> CmdArgs {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        println!("ERROR: 3 args are required");
        panic!("USAGE: ./daemon <address:port> <port_to_redirect> <retry_delay>");
    }
    CmdArgs {
        server_addr: args[1]
            .parse()
            .expect(&format!("ERROR: parsing the address: {}", args[1])),
        local_port: args[2]
            .parse()
            .expect("ERROR: <port_to_redirect> should be a valid port"),
        retry_delay: args[3]
            .parse()
            .expect("ERROR: <retry_delay> should be a valid delay"),
    }
}

fn main() {
    println!("Hello from daemon");
    let args = parse_flags();

    //     loop {
    //         println!("INFO: connecting to tunnel server {} ...", args.server_addr);
    //         match run(&args) {
    //             Err(err) => println!(
    //                 "ERROR: disconnected: {} - retrying in {}s",
    //                 err, args.retry_delay
    //             ),
    //             _ => {}
    //         }
    //         thread::sleep(time::Duration::from_secs(args.retry_delay as u64));
    //     }
}

// fn run(args: &CmdArgs) -> io::Result<()> {
//     let mut tcp_stream = net::TcpStream::connect_timeout(&args.server_addr, TIMEOUT)?;

//     // Send the magic handshake so the server routes this connection to the
//     // tunnel handler instead of the HTTP handler.
//     tcp_stream.set_write_timeout(Some(TIMEOUT))?;
//     let _ = tcp_stream.write(HANDSHAKE.as_bytes())?;

//     let mux = Arc::new(mux::Mux::new(tcp_stream));

//     // The first frame on stream 0 carries our assigned public URL.
//     let public_url = match mux.read_frame() {
//         Ok(first_frame) => String::from_utf8(first_frame.payload.into_vec()).map_err(|err| {
//             io::Error::new(
//                 io::ErrorKind::Other,
//                 format!("ERROR: failed to read bytes: {}", err),
//             )
//         }),
//         Err(err) => Err(err),
//     }?;
//     let public_url = public_url.trim();

//     println!("INFO: tunnel ready — public URL: {}", public_url);
//     println!(
//         "INFO: ✓ Forwarding {}  →  localhost:{}\n\n",
//         public_url, args.local_port
//     );

//     let mut mux_copy = Arc::clone(&mux);
//     let handle = thread::spawn(move || {
//        	mux_copy.serve()
//     });

//     // 	for {
//     // 		stream, err := mux.Accept()
//     // 		if err != nil {
//     // 			return fmt.Errorf("accept stream: %w", err)
//     // 		}
//     // 		go handleStream(stream, *localPort)
//     // 	}
//     // }
//     Ok(())
// }

// // // handleStream reads an HTTP request from the tunnel stream, forwards it to
// // // localhost:<port>, and writes the HTTP response back into the stream.
// // func handleStream(stream *shared.Stream, port int) {
// // 	defer stream.Close()

// // 	req, err := http.ReadRequest(bufio.NewReader(stream))
// // 	if err != nil {
// // 		log.Printf("[stream %d] parse request: %v", stream.ID(), err)
// // 		return
// // 	}

// // 	localAddr := fmt.Sprintf("localhost:%d", port)
// // 	backend, err := net.DialTimeout("tcp", localAddr, 5*time.Second)
// // 	if err != nil {
// // 		log.Printf("[stream %d] dial local %s: %v", stream.ID(), localAddr, err)
// // 		writeErrorResponse(stream, http.StatusBadGateway,
// // 			fmt.Sprintf("daemon could not reach localhost:%d — is it running?", port))
// // 		return
// // 	}
// // 	defer backend.Close()

// // 	if err := req.Write(backend); err != nil {
// // 		log.Printf("[stream %d] write to backend: %v", stream.ID(), err)
// // 		writeErrorResponse(stream, http.StatusBadGateway, "daemon write error: "+err.Error())
// // 		return
// // 	}

// // 	resp, err := http.ReadResponse(bufio.NewReader(backend), req)
// // 	if err != nil {
// // 		log.Printf("[stream %d] read backend response: %v", stream.ID(), err)
// // 		writeErrorResponse(stream, http.StatusBadGateway, "daemon read error: "+err.Error())
// // 		return
// // 	}
// // 	defer resp.Body.Close()

// // 	if err := resp.Write(stream); err != nil {
// // 		log.Printf("[stream %d] write response to stream: %v", stream.ID(), err)
// // 	}
// // }

// // func writeErrorResponse(w io.Writer, code int, msg string) {
// // 	body := msg + "\n"
// // 	fmt.Fprintf(w,
// // 		"HTTP/1.1 %d %s\r\nContent-Type: text/plain\r\nContent-Length: %d\r\nConnection: close\r\n\r\n%s",
// // 		code, http.StatusText(code), len(body), body,
// // 	)
// // }
// //
