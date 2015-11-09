use std::mem;
use std::ops::{
    Deref, DerefMut};

// This stores the current window
#[allow(raw_pointer_derive)]
#[derive(Debug)]
pub struct SlidingWindowStorage<T> {
    window_size: usize,
    uniquely_owned: bool,
    data: Vec<T>,
    _no_send_sync_marker: *const u8 //TODO remove in favour of negative send and sync impls once they are stable
}

/* FIXME: uncomment this once it gets stable
impl !Send for SlidingWindowStorage {}
impl !Sync for SlidingWindowStorage {}
*/

impl<T> Drop for SlidingWindowStorage<T> {
    fn drop(&mut self) {
        // assert that no Window exists when this is dropped
        assert!(self.uniquely_owned, "SlidingWindowStorage dropped before Window went out of scope")
    }
}

impl<T> SlidingWindowStorage<T> {
    pub fn new(window_size: usize) -> SlidingWindowStorage<T> {
        SlidingWindowStorage {
            window_size: window_size,
            uniquely_owned: true,
            data: Vec::with_capacity(window_size),
            _no_send_sync_marker: 0usize as *const u8
        }
    }

    pub fn from_slice(mut s: Vec<T>, window_size: usize) -> SlidingWindowStorage<T> {
        if s.capacity() < window_size {
            let cap = s.capacity();
            s.reserve_exact(window_size - cap);
        }
        s.clear();

        SlidingWindowStorage {
            window_size: window_size,
            uniquely_owned: true,
            data: s,
            _no_send_sync_marker: 0usize as *const u8
        }
    }

    fn new_window(&mut self) -> Window<T> {
        // assert that the last window went out of scope
        assert!(self.uniquely_owned, "next() called before previous Window went out of scope");

        self.uniquely_owned = false;

        unsafe {
            // this creates a second vec managing the SAME memory as self.data.
            // if the destructor of this vec ever runs it will result in a double free!!
            let illegal_vec_copy = Vec::from_raw_parts(self.data.as_mut_ptr(), self.data.len(), self.data.capacity());
            Window { drop_flag: &mut self.uniquely_owned as *mut bool, data: Some(illegal_vec_copy) }
        }
    }
}

#[allow(raw_pointer_derive)]
#[derive(Debug)]
pub struct Window<T> {
    drop_flag: *mut bool,
    data: Option<Vec<T>> // option needed for destructor, will never be None
}

impl<T> Drop for Window<T> {
    fn drop(&mut self) {
        // set flag to indicate this window was dropped
        unsafe {
            *self.drop_flag = true;
        }

        // don't deallocate the Vec as it exists in SlidingWindowStorage too
        // Option::take replaces self with None and returns the Some(x)
        let illegal_vec = self.data.take().unwrap();
        mem::forget(illegal_vec);
    }
}

// convenience impl &Window<T> => &[T]
impl<T> Deref for Window<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.data.as_ref().unwrap()
    }
}

// convenience impl &mut Window<T> => &mut [T]
impl<T> DerefMut for Window<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.data.as_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct SlidingWindowAdapter<'a, I: Iterator> where <I as Iterator>::Item: 'a {
    iter: I,
    storage: &'a mut SlidingWindowStorage<I::Item>,
}

impl<'a, I: Iterator> SlidingWindowAdapter<'a, I> {
    pub fn new(iter: I, storage: &'a mut SlidingWindowStorage<I::Item>) -> SlidingWindowAdapter<'a, I> {
        SlidingWindowAdapter {
            iter: iter,
            storage: storage,
        }
    }
}

impl<'a, I: Iterator> Iterator for SlidingWindowAdapter<'a, I> {
    type Item = Window<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.storage.data.len() != self.storage.window_size {
            // fill window
            while self.storage.data.len() < self.storage.window_size {
                match self.iter.next() {
                    Some(x) => self.storage.data.push(x),
                    None    => break
                }
            }
        } else {
            // remove first element and push next one
            match self.iter.next() {
                Some(x) => {
                    self.storage.data.remove(0);
                    self.storage.data.push(x);
                },
                None => return None
            }
        }

        // return new window
        Some(self.storage.new_window())
    }
}
