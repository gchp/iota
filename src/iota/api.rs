use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::process::exit;
use std::path::PathBuf;

use serde_json;
use libc;

use serde_json::builder::ObjectBuilder;

use ::buffer::Buffer;

#[derive(Debug)]
pub enum Response {
    Empty,
    Integer(usize),
    List(Vec<HashMap<String, String>>),
    
    Error(&'static str)
}


pub struct ServerApi {
    buffers: Vec<Buffer>,
}

impl ServerApi {
    pub fn new() -> ServerApi {
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

    pub fn handle_rpc(&mut self, command: String, args: serde_json::Value) -> Response {
        match &*command {
            "create_buffer" => self.create_buffer(args),
            "list_buffers" => self.list_buffers(),

            _ => Response::Error("Unknown command")
        }
    }
}



