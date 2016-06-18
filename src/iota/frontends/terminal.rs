use std::net::SocketAddr;
use std::process::exit;
use std::io::{Read, Write};

use serde_json;
use serde_json::builder::ObjectBuilder;
use mio::tcp::{Shutdown, TcpStream};
use mio::{TryRead, TryWrite};
use rustbox::{RustBox, InitOptions, InputMode, Event, Key};

use ::uibuf::{UIBuffer, CharStyle};
use ::frontends::CharColor;
use ::frontends::rb::{get_color, get_style};

struct ClientApi {
    stream: TcpStream,
}

impl ClientApi {
    fn new() -> ClientApi {
        let address = "0.0.0.0:10000".parse::<SocketAddr>().unwrap();
        let stream = match TcpStream::connect(&address) {
            Ok(s) => s,
            Err(e) => {
                println!("Error connecting to Iota server: {}", e);
                exit(1);
            }
        };

        ClientApi {
            stream: stream,
        }
    }

    fn server_shutdown(&mut self) {
        let object = ObjectBuilder::new()
            .insert("command", "exit")
            .insert("args", "{}")
            .unwrap();

        let payload = serde_json::to_string(&object).unwrap();
        match self.stream.try_write(payload.as_bytes()) {
            Ok(Some(len)) => {
                loop {
                    let mut result = [0; 2048];
                    match self.stream.try_read(&mut result) {
                        Err(e) => {
                            println!("Error reading socket: {:?}", e);
                            break
                        }
                        Ok(None) => {
                        }
                        Ok(Some(len)) => {
                            break
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn shutdown(&mut self) {
        self.stream.shutdown(Shutdown::Both);
    }

    fn list_buffers(&mut self) -> Vec<String> {
        let object = ObjectBuilder::new()
            .insert("command", "list_buffers")
            .insert("args", "{}")
            .unwrap();

        let mut buffers = Vec::new();

        let payload = serde_json::to_string(&object).unwrap();

        // TODO: refactor
        match self.stream.try_write(payload.as_bytes()) {
            Ok(Some(len)) => {
                loop {
                    let mut result = [0; 2048];
                    match self.stream.try_read(&mut result) {
                        Err(e) => {
                            println!("Error reading socket: {:?}", e);
                            break
                        }
                        Ok(None) => {
                        }
                        Ok(Some(len)) => {
                            let response: serde_json::Value = serde_json::from_slice(&result[0..len]).unwrap();
                            let obj = response.as_object().unwrap();
                            let result = obj.get("result").unwrap();
                            let list = result.as_array().unwrap();

                            for item in list {
                                let obj = item.as_object().unwrap();
                                let path = obj.get("path").unwrap().as_string().unwrap().into();
                                buffers.push(path);
                            }

                            break
                        }
                    }
                }
            }
            _ => {}
        }

        buffers
    }
}

struct TerminalFrontend {
    engine: RustBox,
    ui_buffer: UIBuffer,
    api: ClientApi,


    // TODO: remove this
    current_buffer_path: String,
}


impl TerminalFrontend {

    fn new(engine: RustBox) -> TerminalFrontend {
        let height = engine.height();
        let width = engine.width();

        TerminalFrontend {
            ui_buffer: UIBuffer::new(width, height),
            engine: engine,
            api: ClientApi::new(),

            // TODO: remove this
            current_buffer_path: String::new(),
        }
    }

    fn main_loop(&mut self) { 
        loop {
            self.draw();
            self.engine.present();

            if let Ok(event) = self.engine.poll_event(false) {
                match event {
                    Event::KeyEvent(key) => {
                        match key {
                            Key::Ctrl('q') => { break }
                            _ => {}
                        }
                    }

                    _ => {}
                }
            }
        }
    }


    fn draw(&mut self) {
        let buffer_height = self.ui_buffer.get_height();
        let height = self.engine.height();
        let width = self.engine.width();

        // drawing text in the status bar
        let chars: Vec<char> = self.current_buffer_path.chars().collect();
        for index in 0..chars.len() {
            self.ui_buffer.update_cell(index, height-2, chars[index], CharColor::Black, CharColor::Blue);
        }

        // drawing the rest of the status bar
        for index in self.current_buffer_path.len()..width {
            self.ui_buffer.update_cell(index, height - 2, ' ', CharColor::Black, CharColor::Blue);
        }

        let rows = &mut self.ui_buffer.rows[0..buffer_height];
        for row in rows.iter_mut() {
            for cell in row.iter_mut().filter(|cell| cell.dirty) {
                let bg = get_color(cell.bg);
                let fg = get_color(cell.fg);
                let style = get_style(CharStyle::Normal);

                self.engine.print_char(cell.x, cell.y, style, fg, bg, cell.ch);
                cell.dirty = false;
            }
        }
    }

    fn get_initial_state(&mut self) {
        let buffers = self.api.list_buffers();
        if buffers.len() == 0 {
            panic!("No buffers found");
        } else {
            self.current_buffer_path = buffers[0].clone();
        }
    }
}


pub fn start(server_shutdown: bool) {
    // initialise rustbox
    let rb = match RustBox::init(InitOptions{
        buffer_stderr: true,
        input_mode: InputMode::Esc,
    }) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    // initialise the frontend
    let mut frontend = TerminalFrontend::new(rb);

    frontend.get_initial_state();

    frontend.main_loop();

    if server_shutdown {
        frontend.api.server_shutdown();
    }

    frontend.api.shutdown();
}
