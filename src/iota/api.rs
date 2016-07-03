use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::exit;
use std::path::PathBuf;

use serde_json;
use libc;

use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use uuid::Uuid;

use ::buffer::Buffer;

#[derive(Debug)]
pub enum Response {
    Empty,
    Integer(usize),
    List(Vec<HashMap<String, String>>),
    
    Error(&'static str)
}


struct Workspace {
    name: &'static str,
    buffers: HashMap<Uuid, Buffer>,
}

impl Workspace {
    fn new(name: &'static str) -> Workspace {
        Workspace {
            name: name,
            buffers: HashMap::new(),
        }
    }
}


pub struct ServerApi {
    workspaces: HashMap<&'static str, Workspace>,
}

impl ServerApi {
    pub fn new() -> ServerApi {
        let workspace = Workspace::new("default");

        let mut workspaces = HashMap::new();
        workspaces.insert("default", workspace);

        ServerApi {
            workspaces: workspaces,
        }
    }

    /// Get a list of workspaces
    ///
    /// Response should be in the form:
    ///
    ///     [
    ///         {
    ///             "name": "default",
    ///             "buffers": [
    ///                 { "uuid": "UUID", "file_path": "/file/path.txt" },
    ///                 { "uuid": "UUID", "file_path": "/file/path.txt" },
    ///             ]
    ///         },
    ///         {
    ///             ...
    ///         }
    ///     ]
    fn list_workspaces(&mut self) -> Result<Value, &'static str> {
        let mut items = Vec::new();
        for (ws_name, ws) in &self.workspaces {

            let mut buffer_list = Vec::new();
            for (uuid, buffer) in &ws.buffers {
                let mut buf_map = HashMap::new();
                buf_map.insert("uuid", uuid.to_string());
                                            // this thing here is revolting...plz clean up, future me
                println!("{:?}", buffer.file_path);
                buf_map.insert("file_path", buffer.file_path.clone().unwrap().to_str().unwrap().to_string());
                buffer_list.push(buf_map);
            }

           let value = ObjectBuilder::new()
               .insert("name", ws_name)
               .insert("buffers", buffer_list)
               .unwrap();
           items.push(value);
        }

        Ok(Value::Array(items))
    }

    /// Create a new buffer
    ///
    /// This will create a new buffer in the default workspace. 
    fn create_buffer(&mut self, args: serde_json::Value) -> Result<Value, &'static str> {
        // TODO: allow for the workspace to be overridden
        let mut workspace_name = "default";

        let kwargs = args.as_object().unwrap();
        if let Some(path) = kwargs.get("path").and_then(|v| v.as_string()) {
            let path = PathBuf::from(path);
            println!("path={:?}", path);
            let buffer = Buffer::from(path);

            match self.workspaces.get_mut(&workspace_name) {
                Some(ref mut workspace) => {
                    workspace.buffers.insert(Uuid::new_v4(), buffer);
                    return Ok(Value::Bool(true))
                }
                None => { return Err("No workspace with that name") }
            }
        }

        Err("Could not create buffer")
    }


    pub fn handle_rpc(&mut self, command: String, args: serde_json::Value) -> Result<Value, &'static str> {
        match &*command {
            "create_buffer" => self.create_buffer(args),
            "list_workspaces" => self.list_workspaces(),

            _ => Err("Unknown command")
        }
    }
}



