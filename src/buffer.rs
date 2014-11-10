use std::mem;
use std::ptr;
use std::io::{File, BufferedReader};

use utils;
use cursor::Cursor;


pub struct Buffer {
    length: uint,
    pub first_line: Link,
    pub last_line: Rawlink,

    pub cursor: Cursor,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            length: 0,
            first_line: None,
            last_line: Rawlink::none(),
            cursor: Cursor::new(),
        }
    }

    pub fn new_from_file(path: &Path) -> Buffer {
        let mut file = BufferedReader::new(File::open(path));
        let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();
        let mut buffer = Buffer::new();

        for line in lines.iter() {
            buffer.add_line(line.clone());
        }

        // FIXME: This doesn't seem like a good idea..
        // Update the `cursor.line` to point to the `buffer.first_line`
        let b = buffer.first_line.clone();
        let cursor = Cursor{x:0, y:0, line: b};
        mem::replace(&mut buffer.cursor, cursor);

        buffer
    }

    pub fn draw_contents(&self) {
        for (index, line) in self.iter_lines().enumerate() {
            utils::draw(index, line.value.clone());
        }
    }

    pub fn push_front_line(&mut self, mut new_head: Box<Line>) {
        match self.first_line {
            None => {
                self.last_line = Rawlink::some(&mut *new_head);
                self.first_line = link_with_prev(new_head, Rawlink::none());
            }
            Some(ref mut head) => {
                new_head.prev = Rawlink::none();
                head.prev = Rawlink::some(&mut *new_head);
                mem::swap(head, &mut new_head);
                head.next = Some(new_head);
            }
        }
        self.length += 1;
    }

    pub fn push_back_line(&mut self, mut new_tail: Box<Line>) {
        match self.last_line.resolve() {
            None => return self.push_front_line(new_tail),
            Some(tail) => {
                self.last_line = Rawlink::some(&mut *new_tail);
                tail.next = link_with_prev(new_tail, Rawlink::some(tail));
            }
        }
        self.length += 1
    }

    pub fn add_line(&mut self, elt: String) {
        self.push_back_line(box Line::new(elt));
    }

    pub fn len(&self) -> uint {
        self.length
    }

    pub fn iter_lines<'a>(&'a self) -> Items<'a> {
        Items { nelem: self.len(), head: &self.first_line, tail: self.last_line }
    }
}

fn link_with_prev(mut next: Box<Line>, prev: Rawlink) -> Link {
    next.prev = prev;
    Some(next)
}

struct Items<'a> {
    head: &'a Link,
    tail: Rawlink,
    nelem: uint,
}

impl<'a> Iterator<&'a Line> for Items<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a Line> {
        if self.nelem == 0 {
            return None;
        }
        self.head.as_ref().map(|head| {
            self.nelem -= 1;
            self.head = &head.next;
            &**head
        })
    }
}

#[deriving(Clone)]
pub struct Line {
    pub next: Link,
    pub prev: Rawlink,
    value: String,
}

impl Line {
    pub fn new(v: String) -> Line {
        Line{value: v, next: None, prev: Rawlink::none()}
    }

    pub fn len(&self) -> uint {
        self.value.len()
    }
}

pub type Link = Option<Box<Line>>;

#[deriving(Clone)]
struct Rawlink {
    p: *mut Line,
}

impl Rawlink {
    fn none() -> Rawlink {
        Rawlink{p: ptr::null_mut()}
    }

    fn some(n: &mut Line) -> Rawlink {
        Rawlink{p: n}
    }

    pub fn resolve<'a>(&mut self) -> Option<&'a mut Line> {
        if self.p.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute(self.p) })
        }
    }
}
