use std::ops::Index;

#[derive(Debug, Clone)]
pub struct LazyBuffer<I: Iterator> {
    it: I,
    done: bool,
    buffer: Vec<I::Item>,
}

impl<I> LazyBuffer<I>
where
    I: Iterator,
{
    pub fn new(it: I) -> LazyBuffer<I> {
        let mut it = it;
        let mut buffer = Vec::new();
        let done;
        if let Some(first) = it.next() {
            buffer.push(first);
            done = false;
        } else {
            done = true;
        }
        LazyBuffer {
            it: it,
            done: done,
            buffer: buffer,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn get_next(&mut self) -> bool {
        if self.done {
            return false;
        }
        let next_item = self.it.next();
        match next_item {
            Some(x) => {
                self.buffer.push(x);
                true
            }
            None => {
                self.done = true;
                false
            }
        }
    }
}

impl<I> Index<usize> for LazyBuffer<I>
where
    I: Iterator,
    I::Item: Sized,
{
    type Output = I::Item;

    fn index<'b>(&'b self, _index: usize) -> &'b I::Item {
        self.buffer.index(_index)
    }
}
