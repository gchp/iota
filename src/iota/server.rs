use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::process::exit;
use std::path::PathBuf;

use serde_json;
use libc;

use serde_json::builder::ObjectBuilder;

use mio::*;
use mio::tcp::*;

use ::buffer::Buffer;


const SERVER_TOKEN: Token = Token(0);


#[derive(Debug)]
enum Response {
    Empty,
    Integer(usize),
    List(Vec<HashMap<String, String>>),
    
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

    fn create_buffer(&mut self, args: serde_json::Value) -> Response {
        if let Some(kwargs) = args.as_object() {
            if let Some(path) = kwargs.get("path").and_then(|v| v.as_string()) {
                let path = PathBuf::from(path);
                let buffer = Buffer::from(path);
                self.buffers.push(buffer);
                return Response::Empty
            }
        } 

        let buffer = Buffer::new();
        self.buffers.push(buffer);

        Response::Empty
    }

    fn list_buffers(&mut self) -> Response {
        Response::Integer(self.buffers.len());

        let mut items = Vec::new();

        for buf in self.buffers.iter() {
            let mut map = HashMap::new();

            let path: String = match buf.file_path {
                Some(ref p) => p.to_str().unwrap().to_string(),
                None => String::new(),
            };

            map.insert(String::from("path"), path);
            map.insert(String::from("size"), buf.len().to_string());
            items.push(map);
        }

        Response::List(items)
    }

    fn handle_rpc(&mut self, command: String, args: serde_json::Value) -> Response {
        match &*command {
            "create_buffer" => self.create_buffer(args),
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
        println!("events={:?}", events);
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

                _ => {
                    println!("{:?}", token);
                    let mut client = self.clients.get_mut(&token).unwrap();
                    if let Some((command, args)) = client.read() {
                        println!("command: {}", command);
                        let result = self.api.handle_rpc(command, args);
                        println!("result: {:?}", result);
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
                    panic!("Error while reading socket: {:?}", e);
                }
                Ok(None) => {
                    println!("Noooone");
                    return None
                }
                Ok(Some(0)) => {
                    self.interest.remove(EventSet::readable());
                    self.interest.insert(EventSet::writable());
                    return None
                }
                Ok(Some(len)) =>  {
                    println!("read {} bytes", len);
                    let raw: serde_json::Value = match serde_json::from_slice(&buf[0..len]) {
                        Ok(val) => val,
                        Err(e) => {
                            println!("{:?}", e);
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

