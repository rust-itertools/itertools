use core::mem::{self, MaybeUninit};
use core::ptr;

/// An array of at most `N` elements.
struct ArrayBuilder<T, const N: usize> {
    /// The (possibly uninitialized) elements of the `ArrayBuilder`.
    ///
    /// # Safety
    ///
    /// The elements of `arr[..len]` are valid `T`s.
    arr: [MaybeUninit<T>; N],

    /// The number of leading elements of `arr` that are valid `T`s, len <= N.
    len: usize,
}

impl<T, const N: usize> ArrayBuilder<T, N> {
    /// Initializes a new, empty `ArrayBuilder`.
    pub fn new() -> Self {
        // SAFETY: the validity invariant trivially hold for a zero-length array.
        Self {
            arr: [(); N].map(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }

    /// Pushes `value` onto the end of the array.
    ///
    /// # Panics
    ///
    /// This panics if `self.len() >= N`.
    pub fn push(&mut self, value: T) {
        // SAFETY: we maintain the invariant here that arr[..len] is valid.
        // Indexing with self.len also ensures self.len < N, and thus <= N after
        // the increment.
        self.arr[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    /// Consumes the elements in the `ArrayBuilder` and returns them as an array `[T; N]`.
    ///
    /// If `self.len() < N`, this returns `None`.
    pub fn take(&mut self) -> Option<[T; N]> {
        if self.len == N {
            // Take the array, resetting our length back to zero.
            self.len = 0;
            let arr = mem::replace(&mut self.arr, [(); N].map(|_| MaybeUninit::uninit()));

            // SAFETY: we had len == N, so all elements in arr are valid.
            Some(unsafe { arr.map(|v| v.assume_init()) })
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for ArrayBuilder<T, N> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: arr[..len] is valid, so must be dropped. First we create
            // a pointer to this valid slice, then drop that slice in-place.
            // The cast from *mut MaybeUninit<T> to *mut T is always sound by
            // the layout guarantees of MaybeUninit.
            let ptr_to_first: *mut MaybeUninit<T> = self.arr.as_mut_ptr();
            let ptr_to_slice = ptr::slice_from_raw_parts_mut(ptr_to_first.cast::<T>(), self.len);
            ptr::drop_in_place(ptr_to_slice);
        }
    }
}

/// Equivalent to `it.next_array()`.
pub fn next_array<I, T, const N: usize>(it: &mut I) -> Option<[T; N]>
where
    I: Iterator<Item = T>,
{
    let mut builder = ArrayBuilder::new();
    for _ in 0..N {
        builder.push(it.next()?);
    }
    builder.take()
}
