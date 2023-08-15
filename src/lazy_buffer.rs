use std::iter::Fuse;
use std::ops::Index;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct LazyBuffer<I: Iterator> {
    pub it: Fuse<I>,
    buffer: Vec<I::Item>,
}

impl<I> LazyBuffer<I>
where
    I: Iterator,
{
    pub fn new(it: I) -> LazyBuffer<I> {
        LazyBuffer {
            it: it.fuse(),
            buffer: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_next(&mut self) -> bool {
        if let Some(x) = self.it.next() {
            self.buffer.push(x);
            true
        } else {
            false
        }
    }

    pub fn prefill(&mut self, len: usize) {
        let buffer_len = self.buffer.len();
        if len > buffer_len {
            let delta = len - buffer_len;
            self.buffer.extend(self.it.by_ref().take(delta));
        }
    }
}

impl<I, J> Index<J> for LazyBuffer<I>
where
    I: Iterator,
    I::Item: Sized,
    Vec<I::Item>: Index<J>
{
    type Output = <Vec<I::Item> as Index<J>>::Output;

    fn index(&self, index: J) -> &Self::Output {
        self.buffer.index(index)
    }
}
