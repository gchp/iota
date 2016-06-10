use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::process::exit;

use serde_json;
use libc;

use serde_json::builder::ObjectBuilder;

use mio::*;
use mio::tcp::*;

use ::buffer::Buffer;


const SERVER_TOKEN: Token = Token(0);


enum Response {
    Empty,
    Integer(usize),
    
    Error(&'static str)
}


struct ServerApi {
    buffers: Vec<Buffer>,
}

impl ServerApi {
    fn new() -> ServerApi {
        ServerApi {
            buffers: Vec::new(),
        }
    }

    fn create_buffer(&mut self) -> Response {
        let buffer = Buffer::new();
        self.buffers.push(buffer);

        Response::Empty
    }

    fn list_buffers(&mut self) -> Response {
        Response::Integer(self.buffers.len())
    }

    fn handle_rpc(&mut self, command: String, args: serde_json::Value) -> Response {
        match &*command {
            "create_buffer" => self.create_buffer(),
            "list_buffers" => self.list_buffers(),

            _ => Response::Error("Unknown command")
        }
    }
}


struct Iota {
    socket: TcpListener,
    clients: HashMap<Token, IotaClient>,
    token_counter: usize,
    api: ServerApi,
}


impl Handler for Iota {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Iota>, token: Token, events: EventSet) {
        if events.is_readable() {
            match token {
                SERVER_TOKEN => {
                    let client_socket = match self.socket.accept() {
                        Err(e) => {
                            println!("Accept error: {}", e);
                            return;
                        }
                        Ok(None) => unreachable!("Accept returned 'None'"),
                        Ok(Some((sock, addr))) => sock
                    };

                    self.token_counter += 1;
                    let new_token = Token(self.token_counter);

                    self.clients.insert(new_token, IotaClient::new(client_socket));
                    event_loop.register(&self.clients[&new_token].socket,
                                        new_token, EventSet::readable(),
                                        PollOpt::edge() | PollOpt::oneshot()).unwrap();
                }

                token => {
                    let mut client = self.clients.get_mut(&token).unwrap();
                    if let Some((command, args)) = client.read() {
                        let result = self.api.handle_rpc(command, args);
                        client.result = Some(result);
                    }
                    event_loop.reregister(&client.socket, token, client.interest, PollOpt::edge() | PollOpt::oneshot()).unwrap();
                }
            }
        }

        if events.is_writable() {
            let mut client = self.clients.get_mut(&token).unwrap();
            client.write();
            event_loop.reregister(&client.socket, token, client.interest, PollOpt::edge() | PollOpt::oneshot()).unwrap();
        }
    }
}


struct IotaClient {
    socket: TcpStream,
    interest: EventSet,
    result: Option<Response>,
}

impl IotaClient {
    fn new(socket: TcpStream) -> IotaClient {
        IotaClient {
            socket: socket,
            interest: EventSet::readable(),
            result: None,
        }
    }
    
    fn read(&mut self) -> Option<(String, serde_json::Value)> {
        loop {
            let mut buf = [0; 2048];

            match self.socket.try_read(&mut buf) {
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return None
                }
                Ok(None) => return None,
                Ok(Some(len)) =>  {
                    let raw: serde_json::Value = serde_json::from_slice(&buf[0..len]).unwrap();

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
        self.socket.try_write(response.as_bytes()).unwrap();

        self.interest.remove(EventSet::writable());
        self.interest.insert(EventSet::readable());
    }
}

pub fn start(do_fork: bool) {
    let mut event_loop = EventLoop::new().unwrap();

    let address = "0.0.0.0:10000".parse::<SocketAddr>().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();

    let mut server = Iota {
        token_counter: 1,
        clients: HashMap::new(),
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

