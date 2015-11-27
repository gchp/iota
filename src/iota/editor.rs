use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::VecDeque;

use rustbox;

use input::Input;
use keyboard::Key;
use view::View;
use frontends::EditorEvent;
use buffer::Buffer;
use keymap::KeyMap;
use keymap::KeyMapState;

use textobject::TextObject;
use buffer::Mark;
use textobject::Kind;
use textobject::Offset;
use uibuf::UIBuffer;


#[derive(Copy, Clone, Debug)]
struct Event {
    name: &'static str,
}

impl Event {
    pub fn new(name: &'static str) -> Event {
        Event {
            name: name,
        }
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }
}

/// The main Editor structure
///
/// This is the top-most structure in Iota.
pub struct Editor {
    pub running: bool,

    view: View,
    buffers: Vec<Arc<Mutex<Buffer>>>,
    // mode: Box<Mode + 'e>,
    events_queue: VecDeque<Event>,
    keymap: KeyMap<Event>,
}

impl Editor {
    /// Create a new Editor instance from the given source
    pub fn new(source: Input, width: usize, height: usize) -> Editor {
        let mut buffers = Vec::new();
        let buffer = Buffer::from(source);

        buffers.push(Arc::new(Mutex::new(buffer)));

        let view = View::new(buffers[0].clone(), width, height);
        let mut editor = Editor {
            buffers: buffers,
            view: view,
            running: true,
            // mode: mode,
            events_queue: VecDeque::new(),
            keymap: KeyMap::new(),
        };
        editor.register_key_bindings();

        editor
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: Option<Key>) {
        let key = match key {
            Some(k) => k,
            None => return
        };

        // look up KeyMap
        match self.keymap.check_key(key) {
            KeyMapState::Match(c) => {
                // found a match!
                self.fire_event(c);
            },
            KeyMapState::Continue => {
                // possibly the start of a match...
                // not sure what to do here...
            }
            KeyMapState::None => {
                // no match at all :(
                //
                // lets try insert it into the buffer
                // TODO: use an event for this instead
                if let Key::Char(ch) = key {
                    self.view.insert_char(ch);
                }
            }
        }
    }

    /// Handle resize events
    ///
    /// width and height represent the new height of the window.
    fn handle_resize_event(&mut self, width: usize, height: usize) {
        self.view.resize(width, height);
    }

    /// Draw the current view to the frontend
    pub fn draw(&mut self) {
        self.view.draw();
    }

    fn register_key_bindings(&mut self) {
        // TODO:
        //   Load these default from a JSON file of some sort
        self.bind_keys("up", "iota.move_up");
        self.bind_keys("down", "iota.move_down");
        self.bind_keys("left", "iota.move_left");
        self.bind_keys("right", "iota.move_right");
        self.bind_keys("ctrl-q", "iota.quit");

        self.bind_keys("backspace", "iota.delete_backwards");
        self.bind_keys("delete", "iota.delete_forwards");
        self.bind_keys("enter", "iota.newline");

        self.bind_keys("ctrl-z", "iota.undo");
        self.bind_keys("ctrl-r", "iota.redo");
        self.bind_keys("ctrl-s", "iota.save");
    }

    /// Bind a key to an event
    pub fn bind_keys(&mut self, key_str: &'static str, event: &'static str) {
        // TODO:
        //   it would be nice in the future to be able to store multiple events
        //   for each key. So for instance if an extension was to override a core
        //   keybinding, it would still store the core binding, but mark it as "inactive"
        //   This would allow us to visualize what order event/key bindings are being
        //   stored internally, and potentially disable/enable bindings at will.
        //   As it is now, binding an event to an already bound key will just override
        //   the binding.

        let bits: Vec<&str> = key_str.split(' ').collect();
        let mut keys: Vec<Key> = Vec::new();
        for part in bits {
            keys.push(Key::from(part));
        }
        self.keymap.bind_keys(&*keys, Event::new(event));
    }

    fn fire_event(&mut self, event: Event) {
        self.events_queue.push_back(event);
    }

    fn process_event(&mut self, event: Event) {
        // TODO:
        //   try process event in extensions first
        //   fall back here as a default
        //
        // NOTE:
        //   Extensions should be able to specify in their return
        //   type whether we should also perform the default Action
        //   for an event. For example, if the extension handles the "iota.save"
        //   event, they should be able to tell iota to perform the save,
        //   after whatever custom work they have done. This could be
        //   linting the file, for example.

        match event.get_name() {
            "iota.quit" => { self.running = false; }

            "iota.undo" => { self.view.undo(); }
            "iota.redo" => { self.view.redo(); }
            "iota.save" => { self.view.try_save_buffer(); }

            "iota.newline" => { self.view.insert_char('\n'); }

            "iota.delete_backwards" => {
                self.view.delete_from_mark_to_object(Mark::Cursor(0), TextObject{
                    kind: Kind::Char,
                    offset: Offset::Backward(1, Mark::Cursor(0))
                })
            }
            "iota.delete_forwards" => {
                self.view.delete_from_mark_to_object(Mark::Cursor(0), TextObject{
                    kind: Kind::Char,
                    offset: Offset::Forward(1, Mark::Cursor(0))
                })
            }

            "iota.move_up" => { self.view.move_up() }
            "iota.move_down" => { self.view.move_down() }
            "iota.move_left" => { self.view.move_left() }
            "iota.move_right" => { self.view.move_right() }

            _ => {}
        }
    }

    pub fn start_event_loop(&mut self) {
        while let Some(event) = self.events_queue.pop_front() {
            self.process_event(event);
        }
    }

    /// Start Iota!
    pub fn start(&mut self) {
        self.register_key_bindings();


        while let Some(event) = self.events_queue.pop_front() {
            self.process_event(event);
        }
    }

    pub fn handle_raw_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::KeyEvent(key)         => self.handle_key_event(key),
            EditorEvent::Resize(width, height) => self.handle_resize_event(width, height),

            _ => {}
        }
    }

    pub fn get_cursor_pos(&mut self) -> Option<(isize, isize)> {
        self.view.get_cursor_pos()
    }

    pub fn get_content(&mut self) -> &mut UIBuffer {
        self.view.get_uibuf()
    }

}
