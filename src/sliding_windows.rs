use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};
use std::fmt;

// This stores the current window
pub struct SlidingWindowStorage<T> {
    window_size: usize,
    /// acts as a refcount
    uniquely_owned: Cell<bool>,
    data: UnsafeCell<Vec<T>>
}

/* FIXME: uncomment this once it gets stable
impl !Send for SlidingWindowStorage {}
impl !Sync for SlidingWindowStorage {}
*/

impl<T> SlidingWindowStorage<T> {
    pub fn new(window_size: usize) -> SlidingWindowStorage<T> {
        SlidingWindowStorage {
            window_size: window_size,
            uniquely_owned: Cell::new(true),
            data: UnsafeCell::new(Vec::with_capacity(window_size))
        }
    }

    fn new_window<'a>(&'a self) -> Window<'a, T> {
        // assert that the last window went out of scope
        assert!(self.uniquely_owned.get(), "next() called before previous Window went out of scope");

        self.uniquely_owned.set(false);

        Window { drop_flag: &self.uniquely_owned, data: &self.data }
    }

    fn push(&self, elt: T) -> bool {
        assert!(self.uniquely_owned.get(), "next() called before previous Window went out of scope");
        let data = unsafe { &mut *self.data.get() };
        if data.len() != self.window_size {
            data.push(elt);
        } else {
            data.remove(0);
            data.push(elt);
        }
        data.len() == self.window_size
    }
}

pub struct Window<'a, T: 'a> {
    drop_flag: &'a Cell<bool>,
    data: &'a UnsafeCell<Vec<T>>,
}

impl<'a, T> fmt::Debug for Window<'a, T> where T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self[..].fmt(f)
    }
}

impl<'a, T> Drop for Window<'a, T> {
    fn drop(&mut self) {
        // set flag to indicate this window was dropped
        self.drop_flag.set(true);
    }
}

// convenience impl &Window<T> => &[T]
impl<'a, T> Deref for Window<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        debug_assert!(!self.drop_flag.get());
        unsafe {
            &**self.data.get()
        }
    }
}

// convenience impl &mut Window<T> => &mut [T]
impl<'a, T> DerefMut for Window<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        debug_assert!(!self.drop_flag.get());
        unsafe {
            &mut **self.data.get()
        }
    }
}

impl<'a, 'b, T> PartialEq<&'b [T]> for Window<'a, T> where T: PartialEq
{
    fn eq(&self, other: &&'b [T]) -> bool {
        self[..] == **other
    }
}

pub struct SlidingWindowAdapter<'a, I: Iterator> where <I as Iterator>::Item: 'a {
    iter: I,
    done: bool,
    storage: &'a SlidingWindowStorage<I::Item>,
}

impl<'a, I: Iterator> SlidingWindowAdapter<'a, I> {
    pub fn new(iter: I, storage: &'a SlidingWindowStorage<I::Item>) -> SlidingWindowAdapter<'a, I> {
        SlidingWindowAdapter {
            iter: iter,
            done: false,
            storage: storage,
        }
    }
}

impl<'a, I: Iterator> Iterator for SlidingWindowAdapter<'a, I> {
    type Item = Window<'a, I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None
        }
        self.done = true;
        for elt in &mut self.iter {
            if self.storage.push(elt) {
                self.done = false;
                break;
            }
        }
        if !self.done {
            // return new window
            Some(self.storage.new_window())
        } else {
            None
        }
    }
}
