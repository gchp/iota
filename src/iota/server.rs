use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::process::exit;
use std::path::PathBuf;

use serde_json;
use libc;

use serde_json::builder::ObjectBuilder;

use mio::*;
use mio::tcp::*;
use mio::util::Slab;

use ::buffer::Buffer;
use ::api::{Response, ServerApi};


const SERVER_TOKEN: Token = Token(0);


struct Iota {
    socket: TcpListener,
    clients: Slab<IotaClient>,
    token_counter: usize,
    api: ServerApi,
}


impl Handler for Iota {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Iota>, token: Token, events: EventSet) {
        println!("socket is ready; token={:?}; events={:?}", token, events);
        match token {
            SERVER_TOKEN => {
                assert!(events.is_readable());

                match self.socket.accept() {
                    Err(e) => {
                        println!("Accept error: {}", e);
                        event_loop.shutdown();
                    }
                    Ok(Some((socket, addr))) => {
                        println!("Accepted a new client");
                        let token = self.clients
                            .insert_with(|token| IotaClient::new(socket))
                            .unwrap();

                        event_loop.register(&self.clients[token].socket,
                                            token, EventSet::readable(),
                                            PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    }
                    Ok(None) => {
                        println!("socket wasn't actually available")
                    }
                }
            }

            _ => {
                if events.is_readable() {
                    println!("read_token={:?}", token);
                    if let Some((command, args)) = self.clients[token].read() {
                        println!("command: {}", command);
                        let result = self.api.handle_rpc(command, args);
                        println!("result: {:?}", result);
                        self.clients[token].result = Some(result);
                    }
                    if self.clients[token].closed {
                        self.clients.remove(token);
                    } else {
                        event_loop.reregister(&self.clients[token].socket, token, self.clients[token].interest, PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    }
                }
                if events.is_writable() {
                    println!("write_token={:?}", token);
                    self.clients[token].write();
                    if self.clients[token].closed {
                        self.clients.remove(token);
                    } else {
                        event_loop.reregister(&self.clients[token].socket, token, self.clients[token].interest, PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    }
                }
            }
        }

    }
}


struct IotaClient {
    socket: TcpStream,
    interest: EventSet,
    result: Option<Response>,
    closed: bool,
}

impl IotaClient {
    fn new(socket: TcpStream) -> IotaClient {
        IotaClient {
            socket: socket,
            interest: EventSet::readable(),
            result: None,
            closed: false,
        }
    }
    
    fn read(&mut self) -> Option<(String, serde_json::Value)> {
        let mut buf = [0; 2048];

        // NOTE: not sure if this all needs to be inside a loop...

        match self.socket.try_read(&mut buf) {
            Err(e) => {
                panic!("Error while reading socket: {:?}", e);
            }
            Ok(None) => {
                println!("Noooone");
                return None
            }
            Ok(Some(0)) => {
                println!("Got zero");
                self.closed = true;
                self.interest.remove(EventSet::readable());
                self.interest.insert(EventSet::writable());
                return None
            }
            Ok(Some(len)) =>  {
                println!("read {} bytes", len);
                println!("{}", String::from_utf8(Vec::from(&buf[0..len])).unwrap());
                let raw: serde_json::Value = match serde_json::from_slice(&buf[0..len]) {
                    Ok(val) => val,
                    Err(e) => {
                        println!("Error parsing as JSON {}", e);
                        return None
                    }
                };

                self.interest.remove(EventSet::readable());
                self.interest.insert(EventSet::writable());

                if let Some(obj) = raw.as_object() {
                    if let (Some(method), Some(args)) = (obj.get("command").and_then(|v| v.as_string()), obj.get("args")) {
                        // TODO: don't clone args here
                        return Some((String::from(method), args.clone()))
                    }
                }
                return None
            }
        }
    }

    fn write(&mut self) {
        let builder = match self.result {
            Some(ref response)    => {
                match response {
                    &Response::Empty => {
                        ObjectBuilder::new()
                            .insert("response", "ok")
                            .unwrap()
                    }

                    &Response::Integer(val) => {
                        ObjectBuilder::new()
                            .insert("response", "ok")
                            .insert("result", val)
                            .unwrap()
                    }

                    &Response::List(ref list) => {
                        ObjectBuilder::new()
                            .insert("response", "ok")
                            .insert("result", list)
                            .unwrap()
                    }

                    &Response::Error(msg) => {
                        ObjectBuilder::new()
                            .insert("response", "error")
                            .insert("message", msg)
                            .unwrap()
                    }
                }
            }
            None => {
                panic!("Why did this happen?")
            }
        };

        let response = serde_json::to_string(&builder).unwrap();
        println!("response={}", response);
        match self.socket.try_write(response.as_bytes()) {
            Ok(Some(0)) => {
            }
            Ok(Some(n)) => {
                println!("Wrote {} bytes", n);
                self.interest.remove(EventSet::writable());
                self.interest.insert(EventSet::readable());
            }

            Ok(None) => {
                println!("not ready for writing");
                return 
            }

            Err(e) => {
                println!("interest: {:?}", self.interest);
                panic!("Error in writing: {}", e);
            }

        }

    }
}

pub fn start(do_fork: bool) {
    let mut event_loop = EventLoop::new().unwrap();

    let address = "0.0.0.0:10000".parse::<SocketAddr>().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();

    let mut server = Iota {
        token_counter: 1,
        clients: Slab::new_starting_at(Token(1), 1024),
        socket: server_socket,
        api: ServerApi::new(),
    };

    event_loop.register(
        &server.socket,
        SERVER_TOKEN,
        EventSet::readable(),
        PollOpt::edge()
    ).unwrap();

    if do_fork {
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            exit(-1)
        }
        if pid != 0 {
            exit(0)
        }
    }

    event_loop.run(&mut server).unwrap();
}

