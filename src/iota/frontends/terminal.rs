use std::net::SocketAddr;
use std::process::exit;
use std::io::{Read, Write};

use serde_json;
use serde_json::builder::ObjectBuilder;
use mio::tcp::TcpStream;
use mio::{TryRead, TryWrite};
use rustbox::{RustBox, InitOptions, InputMode};

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

    fn create_buffer(&mut self) {
        let builder = ObjectBuilder::new()
            .insert("command", "create_buffer")
            .insert("args", "{}")
            .unwrap();
        let payload = serde_json::to_string(&builder).unwrap();

        self.stream.try_write(payload.as_bytes()).unwrap();

        let mut result = [0; 2048];
        match self.stream.try_read(&mut result) {
            Err(e) => { println!("Error reading socket: {:?}", e) }
            Ok(None) => {}
            Ok(Some(len)) => {
                let response: serde_json::Value = serde_json::from_slice(&result[0..len]).unwrap();
                println!("{:?}", response);
            }
        }
    }
}

struct TerminalFrontend {
    engine: RustBox,
    ui_buffer: UIBuffer,
    api: ClientApi,
}


impl TerminalFrontend {

    fn new(engine: RustBox) -> TerminalFrontend {
        let height = engine.height();
        let width = engine.width();

        TerminalFrontend {
            ui_buffer: UIBuffer::new(width, height),
            engine: engine,
            api: ClientApi::new(),
        }
    }

    fn main_loop(&mut self) { 
        loop {
            self.draw();
            self.engine.present();
            let event = self.engine.poll_event(true);
        }
    }


    fn draw(&mut self) {
        let buffer_height = self.ui_buffer.get_height();
        let height = self.engine.height();
        let width = self.engine.width();

        for index in 0..width {
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

    fn set_initial_state(&mut self) {
        self.api.create_buffer(); 
    }
}


pub fn start() {
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

    frontend.set_initial_state();

    frontend.main_loop();
}
