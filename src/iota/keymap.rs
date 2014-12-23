use std::collections::{ HashMap };

use keyboard::Key;
use editor::Command;
use cursor::Direction;

pub enum Trie {
    Leaf(Command),
    Node(HashMap<Key, Box<Trie>>)
}

impl Trie {
    fn new() -> Trie {
        Trie::Node(HashMap::new())
    }
    fn lookup_key(&self, key: Key) -> Option<&Box<Trie>> {
        match *self {
            Trie::Leaf(_) => None,
            Trie::Node(ref map) => map.get(&key)
        }
    }
    fn lookup_keys(&self, keys: &[Key]) -> Option<&Trie> {
        let mut current = self;

        for key in keys.iter() {
            if let Some(node) = current.lookup_key(*key) {
                match **node {
                    Trie::Leaf(_) => return Some(&(**node)),
                    Trie::Node(_) => current = &(**node)
                }
            } else {
                return None
            }
        }

        return Some(&(*current))
    }
    fn bind_key(&mut self, key: Key, command: Command) {
        match *self {
            Trie::Leaf(_) => {
                *self = Trie::new();
                self.bind_key(key, command);
            }
            Trie::Node(ref mut map) => {
                map.insert(key, box Trie::Leaf(command));
            }
        }
    }
    fn bind_keys(&mut self, keys: &[Key], command: Command) {
        if keys.len() == 1 {
            self.bind_key(keys[0], command);
        } else if keys.len() > 1 {
            match *self {
                Trie::Leaf(_) => {
                    *self = Trie::new();
                    self.bind_keys(keys, command);
                }
                Trie::Node(ref mut map) => {
                    let key = keys[0];
                    let keys = keys.slice_from(1);

                    if map.contains_key(&key) {
                        map.get_mut(&key).unwrap().bind_keys(keys, command);
                    } else {
                        let mut node = box Trie::new();
                        node.bind_keys(keys, command);
                        map.insert(key, node);
                    }
                }
            }
        }
    }
}

#[deriving(Copy, Show)]
pub enum KeyMapState {
    Match(Command),     // found a match
    Continue,           // needs another key to disambiguate
    None                // no match
}

pub struct KeyMap {
    root: Trie,
    state: KeyMapState,
    path: Vec<Key>
}

impl KeyMap {
    pub fn new() -> KeyMap {
        KeyMap {
            root: Trie::new(),
            state: KeyMapState::None,
            path: Vec::new()
        }
    }

    /// Eat one keypress, return the new state
    pub fn check_key(&mut self, key: Key) -> KeyMapState {
        self.path.push(key);
        self.state = match self.root.lookup_keys(self.path.as_slice()) {
            Some(n) => {
                match *n {
                    Trie::Leaf(command) => KeyMapState::Match(command),
                    Trie::Node(_) => KeyMapState::Continue
                }
            }
            _ => { self.path.clear(); KeyMapState::None }
        };
        match self.state {
            KeyMapState::Match(command) => {
                self.state = KeyMapState::None;
                self.path.clear();
                return KeyMapState::Match(command)
            },
            _ => self.state
        }
    }

    /// Insert or overwrite a key-sequence binding
    pub fn bind_keys(&mut self, keys: &[Key], command: Command) {
        self.root.bind_keys(keys.as_slice(), command);
    }

    /// Insert or overwrite a key binding
    pub fn bind_key(&mut self, key: Key, command: Command) {
        self.root.bind_key(key, command);
    }

    pub fn load_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), Command::ExitEditor);
        keymap.bind_key(Key::Ctrl('s'), Command::SaveBuffer);
        keymap.bind_key(Key::Ctrl('r'), Command::ResizeView);
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('c')].as_slice(), Command::ExitEditor);
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('s')].as_slice(), Command::SaveBuffer);

        // Navigation
        keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up));
        keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down));
        keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left));
        keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right));

        keymap.bind_key(Key::Ctrl('p'), Command::MoveCursor(Direction::Up));
        keymap.bind_key(Key::Ctrl('n'), Command::MoveCursor(Direction::Down));
        keymap.bind_key(Key::Ctrl('b'), Command::MoveCursor(Direction::Left));
        keymap.bind_key(Key::Ctrl('f'), Command::MoveCursor(Direction::Right));

        keymap.bind_key(Key::Ctrl('e'), Command::LineEnd);
        keymap.bind_key(Key::Ctrl('a'), Command::LineStart);

        // Editing
        keymap.bind_key(Key::Tab, Command::InsertTab);
        keymap.bind_key(Key::Enter, Command::InsertLine);
        keymap.bind_key(Key::Backspace, Command::Delete(Direction::Left));
        keymap.bind_key(Key::Ctrl('h'), Command::Delete(Direction::Left));
        keymap.bind_key(Key::Delete, Command::Delete(Direction::Right));
        keymap.bind_key(Key::Ctrl('d'), Command::Delete(Direction::Right));

        // History
        keymap.bind_key(Key::Ctrl('y'), Command::Redo);
        keymap.bind_key(Key::Ctrl('z'), Command::Undo);

        return keymap
    }
}
