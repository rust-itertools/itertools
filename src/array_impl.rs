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
    pub fn new(iter: I) -> Self {
        Self { iter, buf: ArrayVec::new() }
    }
}
impl<I: Iterator, const N: usize> Arrays<I, N> where I::Item: Clone {
    pub fn remaining(self) -> Vec<I::Item> {
        self.buf.into_vec()
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

pub fn next_array<I: Iterator, const N: usize>(iter: &mut I) -> Option<[I::Item; N]> {
    let mut array_vec = ArrayVec::new();

    // SAFETY:
    // ArrayVec::push_unchecked is safe as long as len < N.
    // This is guaranteed by the for loop
    unsafe {
        for _ in 0..N {
            array_vec.push_unchecked(iter.next()?)
        }
    }

    array_vec.into_array()
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
    pub const fn new() -> Self {
        Self {
            data: std::mem::MaybeUninit::uninit(),
            len: 0,
        }
    }

    pub fn push(&mut self, v: T) {
        assert!(self.len < N);
        // SAFETY: asserts that len < N.
        unsafe { self.push_unchecked(v) }
    }

    // Unsafety:
    // len must be less than N. If `len < N` is guaranteed, this operation is safe
    // This is because the contract of ArrayVec guarantees that if len < N, then the value
    // at len is valid and uninitialised.
    pub unsafe fn push_unchecked(&mut self, v: T) {
        // The contract of ArrayVec guarantees that the value at self.len, if < N,
        // is uninitialised, and therefore does not need dropping. So use write to
        // overwrite the value
        let ptr = (self.data.as_mut_ptr() as *mut T).add(self.len);
        std::ptr::write(ptr, v);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn into_array(self) -> Option<[T; N]> {
        if self.len == N {
            // SAFETY:
            // If len == N, then all the data is initialised and this is safe
            unsafe { Some(self.into_array_unchecked()) }
        } else {
            None
        }
    }

    // Unsafety:
    // len must be equal to N. If `len == N` is guaranteed, this operation is safe.
    // This is because the contract of ArrayVec guarantees that if len == N, all the values
    // have been initialised correctly.
    unsafe fn into_array_unchecked(mut self) -> [T; N] {
        // move out without dropping
        let data = std::mem::replace(&mut self.data, std::mem::MaybeUninit::uninit().assume_init());
        std::mem::forget(self);
        data.assume_init()
    }

    // Unsafety:
    // len must be equal to N. If `len == N` is guaranteed, this operation is safe.
    // This is because the contract of ArrayVec guarantees that if len == N, all the values
    // have been initialised correctly.
    unsafe fn take_unchecked(&mut self) -> [T; N] {
        let data = std::mem::replace(&mut self.data, std::mem::MaybeUninit::uninit().assume_init());
        self.len = 0;
        data.assume_init()
    }
}

impl<T: Clone, const N: usize> ArrayVec<T, N> {
    pub fn into_vec(mut self) -> Vec<T> {
        unsafe {
            let data = std::mem::replace(&mut self.data, std::mem::MaybeUninit::uninit().assume_init());
            let len = self.len;
            std::mem::forget(self);
            std::slice::from_raw_parts(data.as_ptr() as *const T, len).to_vec()
        }
    }
}
