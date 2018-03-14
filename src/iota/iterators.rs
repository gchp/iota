use gapbuffer::GapBuffer;

pub struct Lines<'a> {
    pub buffer: &'a GapBuffer<char>,
    pub tail: usize,
    pub head: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.tail == self.head { return None; }
        let old_tail = self.tail;
        //update tail to either the first char after the next \n or to self.head
        self.tail = (old_tail..self.head).filter(|i| { *i + 1 == self.head
                                                       || self.buffer[*i] == '\n' })
                                         .take(1)
                                         .next()
                                         .unwrap() + 1;
        Some((old_tail..if self.tail == self.head { self.tail - 1 } else { self.tail })
            .map( |i| self.buffer[i] ).collect())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        //TODO: this is technically correct but a better estimate could be implemented
        (1, Some(self.head))
    }

}
