use std::collections::HashMap;
use keyboard::Key;

pub enum Trie<T: Copy> {
    Leaf(T),
    Node(HashMap<Key, Box<Trie<T>>>)
}

impl<T: Copy> Trie<T> {
    fn new() -> Trie<T> {
        Trie::Node(HashMap::new())
    }
    fn lookup_key(&self, key: Key) -> Option<&Box<Trie<T>>> {
        match *self {
            Trie::Leaf(_) => None,
            Trie::Node(ref map) => map.get(&key)
        }
    }
    fn lookup_keys(&self, keys: &[Key]) -> Option<&Trie<T>> {
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
    fn bind_key(&mut self, key: Key, value: T) {
        match *self {
            Trie::Leaf(_) => {
                *self = Trie::new();
                self.bind_key(key, value);
            }
            Trie::Node(ref mut map) => {
                map.insert(key, Box::new(Trie::Leaf(value)));
            }
        }
    }
    fn bind_keys(&mut self, keys: &[Key], value: T) {
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
                    let keys = keys.slice_from(1);

                    if map.contains_key(&key) {
                        map.get_mut(&key).unwrap().bind_keys(keys, value);
                    } else {
                        let mut node = Box::new(Trie::new());
                        node.bind_keys(keys, value);
                        map.insert(key, node);
                    }
                }
            }
        }
    }
}

#[derive(Copy, Show)]
pub enum KeyMapState<T> {
    Match(T),     // found a match
    Continue,     // needs another key to disambiguate
    None          // no match
}

/// Map sequences of `Key`s to values
pub struct KeyMap<T: Copy> {
    root: Trie<T>,
    state: KeyMapState<T>,
    path: Vec<Key>
}

impl<T: Copy> KeyMap<T> {
    pub fn new() -> KeyMap<T> {
        KeyMap {
            root: Trie::new(),
            state: KeyMapState::None,
            path: Vec::new()
        }
    }

    /// Eat one keypress, return the new state
    pub fn check_key(&mut self, key: Key) -> KeyMapState<T> {
        self.path.push(key);
        self.state = match self.root.lookup_keys(self.path.as_slice()) {
            Some(n) => {
                match *n {
                    Trie::Leaf(value) => KeyMapState::Match(value),
                    Trie::Node(_) => KeyMapState::Continue
                }
            }
            _ => { self.path.clear(); KeyMapState::None }
        };
        match self.state {
            KeyMapState::Match(value) => {
                self.state = KeyMapState::None;
                self.path.clear();
                return KeyMapState::Match(value)
            },
            _ => self.state
        }
    }

    /// Insert or overwrite a key-sequence binding
    pub fn bind_keys(&mut self, keys: &[Key], value: T) {
        self.root.bind_keys(keys.as_slice(), value);
    }

    /// Insert or overwrite a key binding
    pub fn bind_key(&mut self, key: Key, value: T) {
        self.root.bind_key(key, value);
    }
}
