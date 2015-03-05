use std::mem;
use gapbuffer::GapBuffer;

pub struct Lines<'a> {
    pub buffer: &'a GapBuffer<u8>,
    pub tail: usize,
    pub head: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        if self.tail != self.head {
            let old_tail = self.tail;
            //update tail to either the first char after the next \n or to self.head
            self.tail = range(old_tail, self.head).filter(|i| { *i + 1 == self.head
                                                                || self.buffer[*i] == b'\n' })
                                                  .take(1)
                                                  .next()
                                                  .unwrap() + 1;
            Some(range(old_tail, if self.tail == self.head { self.tail - 1 } else { self.tail })
                .map( |i| self.buffer[i] ).collect())
        } else { None }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        //TODO: this is technically correct but a better estimate could be implemented
        (1, Some(self.head))
    }

}

/// Iterator traversing GapBuffer as chars
/// Can be made to traverse forwards or backwards with the methods
/// `rev()` `forward()` and `backward()`
pub struct Chars<'a> {
    pub buffer: &'a GapBuffer<u8>,
    pub idx: usize,
    pub forward: bool,
}

/// Iterator traversing GapBuffer as tuples of (buffer_index, char)
/// Can be made to traverse forwards or backwards with the methods
/// `rev()` `forward()` and `backward()`
pub struct CharIndices<'a> {
    iter: Chars<'a>
}

// helper macros/constants
// u8 -> char code is mostly copied from core::str::Chars
const CONT_MASK:   u8 = 0b0011_1111u8;
const TAG_CONT_U8: u8 = 0b1000_0000u8;

// Return the initial codepoint accumulator for the first byte.
// The first byte is special, only want bottom 5 bits for width 2, 4 bits
// for width 3, and 3 bits for width 4
macro_rules! utf8_first_byte {
    ($byte:expr, $width:expr) => (($byte & (0x7F >> $width)) as u32)
}

// return the value of $ch updated with continuation byte $byte
macro_rules! utf8_acc_cont_byte {
    ($ch:expr, $byte:expr) => (($ch << 6) | ($byte & CONT_MASK) as u32)
}

macro_rules! utf8_is_cont_byte {
    ($byte:expr) => (($byte & !CONT_MASK) == TAG_CONT_U8)
}


impl<'a> Chars<'a> {
    pub fn reverse(mut self) -> Chars<'a> {
        self.forward = !self.forward;
        self
    }
    pub fn forward(mut self) -> Chars<'a> {
        self.forward = true;
        self
    }
    pub fn backward(mut self) -> Chars<'a> {
        self.forward = false;
        self
    }

    /// Return an iterator over (byte-index, char)
    pub fn indices(mut self) -> CharIndices<'a> {
        CharIndices {
            iter: self
        }
    }

    fn next_u8(&mut self) -> Option<u8> {
        let n = if self.idx < self.buffer.len() {
            Some(self.buffer[self.idx])
        } else { None };

        if self.forward {
            self.idx += 1;
        } else {
            self.idx -= 1;
        }
        n
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        if self.forward {
            // read u8s forwards into char (largely copied from core::str::Chars)
            let x = match self.next_u8() {
                None => return None,
                Some(next_byte) if next_byte < 128 => return Some(next_byte as char),
                Some(next_byte) => next_byte
            };

            // Multibyte case follows
            // Decode from a byte combination out of: [[[x y] z] w]
            let init = utf8_first_byte!(x, 2);
            let y = self.next_u8().unwrap_or(0);
            let mut ch = utf8_acc_cont_byte!(init, y);
            if x >= 0xE0 {
                // [[x y z] w] case
                // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
                let z = self.next_u8().unwrap_or(0);
                let y_z = utf8_acc_cont_byte!((y & CONT_MASK) as u32, z);
                ch = init << 12 | y_z;
                if x >= 0xF0 {
                    // [x y z w] case
                    // use only the lower 3 bits of `init`
                    let w = self.next_u8().unwrap_or(0);
                    ch = (init & 7) << 18 | utf8_acc_cont_byte!(y_z, w);
                }
            }

            // str invariant says `ch` is a valid Unicode Scalar Value
            return unsafe { Some(mem::transmute(ch)) };
        } else {
            // read u8s backwards into char (largely copied from core::str::Chars)
            let w = match self.next_u8() {
                None => return None,
                Some(back_byte) if back_byte < 128 => return Some(back_byte as char),
                Some(back_byte) => back_byte,
            };

            // Multibyte case follows
            // Decode from a byte combination out of: [x [y [z w]]]
            let mut ch;
            let z = self.next_u8().unwrap_or(0);
            ch = utf8_first_byte!(z, 2);
            if utf8_is_cont_byte!(z) {
                let y = self.next_u8().unwrap_or(0);
                ch = utf8_first_byte!(y, 3);
                if utf8_is_cont_byte!(y) {
                    let x = self.next_u8().unwrap_or(0);
                    ch = utf8_first_byte!(x, 4);
                    ch = utf8_acc_cont_byte!(ch, y);
                }
                ch = utf8_acc_cont_byte!(ch, z);
            }
            ch = utf8_acc_cont_byte!(ch, w);

            // str invariant says `ch` is a valid Unicode Scalar Value
            return unsafe { Some(mem::transmute(ch)) };
        }
    }
}


impl<'a> CharIndices<'a> {
    pub fn rev(mut self) -> CharIndices<'a> {
        self.iter.forward = !self.iter.forward;
        self
    }
    pub fn forward(mut self) -> CharIndices<'a> {
        self.iter.forward = true;
        self
    }
    pub fn backward(mut self) -> CharIndices<'a> {
        self.iter.forward = false;
        self
    }
}

impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<(usize, char)> {
        if let Some(c) = self.iter.next() {
            Some((self.iter.idx, c))
        } else {
            None
        }
    }
}
