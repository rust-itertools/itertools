/// An iterator that groups the items in arrays of a specific size.
///
/// See [`.arrays()`](crate::Itertools::arrays) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Arrays<I: Iterator, const N: usize>
{
    iter: I,
    buf: ArrayVec<I::Item, N>,
}

impl<I: Iterator, const N: usize> Arrays<I, N> {
    /// Creates a new Arrays iterator.
    /// See [`.arrays()`](crate::Itertools::arrays) for more information.
    pub fn new(iter: I) -> Self {
        Self { iter, buf: ArrayVec::new() }
    }
}
impl<I: Iterator, const N: usize> Arrays<I, N> where I::Item: Clone {
    /// Returns any remaining data left in the iterator.
    /// This is useful if the iterator has left over values
    /// that didn't fit into the array size.
    pub fn remaining(&self) -> &[I::Item] {
        self.buf.into_slice()
    }
}

impl<I: Iterator, const N: usize> Iterator for Arrays<I, N> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY:
        // ArrayVec::push_unchecked is safe as long as len < N.
        // This is guaranteed by the for loop
        unsafe {
            for _ in self.buf.len()..N {
                self.buf.push_unchecked(self.iter.next()?)
            }

            Some(self.buf.take_unchecked())
        }
    }
}

/// Reads N elements from the iter, returning them in an array. If there's not enough
/// elements, returns None.
pub fn next_array<I: Iterator, const N: usize>(iter: &mut I) -> Option<[I::Item; N]> {
    let mut array_vec = ArrayVec::new();

    unsafe {
        // SAFETY:
        // ArrayVec::push_unchecked is safe as long as len < N.
        // This is guaranteed by the for loop
        for _ in 0..N {
            array_vec.push_unchecked(iter.next()?)
        }

        // SAFETY:
        // We have guaranteed to have filled all N elements
        Some(array_vec.into_array_unchecked())
    }
}

// ArrayVec is a safe wrapper around a [T; N].
// It allows safely initialising an empty array
pub struct ArrayVec<T, const N: usize> {
    data: std::mem::MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        // SAFETY:
        // The contract of the ArrayVec ensures that data[..len] is initialised
        unsafe {
            let ptr = self.data.as_mut_ptr() as *mut T;
            drop(std::slice::from_raw_parts_mut(ptr, self.len));
        }
    }
}

impl<T, const N: usize> ArrayVec<T, N> {
    /// Creates a new empty ArrayVec
    pub const fn new() -> Self {
        Self {
            data: std::mem::MaybeUninit::uninit(),
            len: 0,
        }
    }

    /// Returns the number of initialised elements in the ArrayVec
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the ArrayVec is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether the ArrayVec is full
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// Push a value into the ArrayVec
    ///
    /// Panics:
    /// If the ArrayVec is full, this function will panic
    pub fn push(&mut self, v: T) {
        assert!(!self.is_full());
        // SAFETY: asserted that self is not full
        unsafe { self.push_unchecked(v) }
    }

    /// Push a value into the ArrayVec
    ///
    /// Unsafety:
    /// The ArrayVec must not be full. If the ArrayVec is full, this function will try write data
    /// out-of-bounds.
    pub unsafe fn push_unchecked(&mut self, v: T) {
        // The contract of ArrayVec guarantees that the value at self.len, if < N,
        // is uninitialised, and therefore does not need dropping. So use write to
        // overwrite the value
        let ptr = (self.data.as_mut_ptr() as *mut T).add(self.len);
        std::ptr::write(ptr, v);
        self.len += 1;
    }

    /// If the ArrayVec is full, returns the data owned. Otherwise, returns None
    pub fn into_array(self) -> Option<[T; N]> {
        if self.is_full() {
            // SAFETY:
            // If len == N, then all the data is initialised and this is safe
            unsafe { Some(self.into_array_unchecked()) }
        } else {
            None
        }
    }

    /// Returns the data owned by the ArrayVec
    ///
    /// Unsafety:
    /// The ArrayVec must be full. If it is not full, some of the values in the array will be
    /// unintialised. This will cause undefined behaviour
    unsafe fn into_array_unchecked(mut self) -> [T; N] {
        // move out without dropping
        let data = std::mem::replace(&mut self.data, std::mem::MaybeUninit::uninit().assume_init());
        std::mem::forget(self);
        data.assume_init()
    }

    /// Returns the data owned by the ArrayVec, resetting the ArrayVec to an empty state
    ///
    /// Unsafety:
    /// The ArrayVec must be full. If it is not full, some of the values in the array will be
    /// unintialised. This will cause undefined behaviour
    unsafe fn take_unchecked(&mut self) -> [T; N] {
        let data = std::mem::replace(&mut self.data, std::mem::MaybeUninit::uninit().assume_init());
        self.len = 0;
        data.assume_init()
    }

    /// Borrows the initialised data in the ArrayVec
    pub fn into_slice(&self) -> &[T] {
        unsafe {
            // let data = std::mem::replace(&mut self.data, std::mem::MaybeUninit::uninit().assume_init());
            // let len = self.len;
            // std::mem::forget(self);
            std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len)
        }
    }
}
