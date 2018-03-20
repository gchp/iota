use std::collections::HashMap;
use std::collections::hash_map::Entry;

use command::{BuilderArgs, Command};
use keyboard::Key;

pub enum Trie {
    Leaf(String),
    Node(HashMap<Key, Trie>)
}

impl Trie {
    fn new() -> Trie {
        Trie::Node(HashMap::new())
    }
    fn lookup_key(&self, key: Key) -> Option<&Trie> {
        match *self {
            Trie::Leaf(_) => None,
            Trie::Node(ref map) => map.get(&key)
        }
    }
    fn lookup_keys(&self, keys: &[Key]) -> Option<&Trie> {
        let mut current = self;

        for key in keys.iter() {
            if let Some(node) = current.lookup_key(*key) {
                match *node {
                    Trie::Leaf(_) => return Some(&(*node)),
                    Trie::Node(_) => current = &(*node)
                }
            } else {
                return None
            }
        }

        Some(&(*current))
    }
    fn bind_key(&mut self, key: Key, value: String) {
        match *self {
            Trie::Leaf(_) => {
                *self = Trie::new();
                self.bind_key(key, value);
            }
            Trie::Node(ref mut map) => {
                map.insert(key, Trie::Leaf(value));
            }
        }
    }
    fn bind_keys(&mut self, keys: &[Key], value: String) {
        if keys.len() == 1 {
            self.bind_key(keys[0], value);
        } else if keys.len() > 1 {
            match *self {
                Trie::Leaf(_) => {
                    *self = Trie::new();
                    self.bind_keys(keys, value);
                }
                Trie::Node(ref mut map) => {
                    let key = keys[0];
                    let keys = &keys[1..];
                    match map.entry(key) {
                        Entry::Vacant(v) => {
                            let mut node = Trie::new();
                            node.bind_keys(keys, value);
                            v.insert(node);
                        },
                        Entry::Occupied(mut o) =>
                            o.get_mut().bind_keys(keys, value)
                    }
                }
            }
        }
    }
}

pub enum KeyMapState {
    Match(String),     // found a match
    Continue,     // needs another key to disambiguate
    None          // no match
}

/// Map sequences of `Key`s to values
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
        self.state = match self.root.lookup_keys(&*self.path) {
            Some(n) => {
                match n {
                    &Trie::Leaf(ref value) => KeyMapState::Match(value.clone()),
                    &Trie::Node(_) => KeyMapState::Continue
                }
            }
            _ => { self.path.clear(); KeyMapState::None }
        };
        let (new_state, ret_val) = match self.state {
            KeyMapState::Match(ref value) => {
                // self.state = KeyMapState::None;
                self.path.clear();
                (Some(KeyMapState::None), KeyMapState::Match(value.to_string()))
            },
            KeyMapState::Continue => (None, KeyMapState::Continue),
            KeyMapState::None => (None, KeyMapState::None),
        };

        if let Some(st) = new_state {
            self.state = st;
        }

        ret_val
    }

    /// Insert or overwrite a key-sequence binding
    pub fn bind_keys(&mut self, keys: &[Key], value: String) {
        self.root.bind_keys(&*keys, value);
    }

    /// Insert or overwrite a key binding
    pub fn bind_key(&mut self, key: Key, value: String) {
        self.root.bind_key(key, value);
    }
}
