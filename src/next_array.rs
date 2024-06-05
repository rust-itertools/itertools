use core::mem::MaybeUninit;
use core::ptr;

/// Helper struct to build up an array element by element.
struct ArrayBuilder<T, const N: usize> {
    arr: [MaybeUninit<T>; N], // Invariant: arr[..len] is initialized.
    len: usize,               // Invariant: len <= N.
}

impl<T, const N: usize> ArrayBuilder<T, N> {
    pub fn new() -> Self {
        Self {
            arr: [(); N].map(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        // We maintain the invariant here that arr[..len] is initialized.
        // Indexing with self.len also ensures self.len < N, and thus <= N after
        // the increment.
        self.arr[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    pub fn take(&mut self) -> Option<[T; N]> {
        if self.len == N {
            // Take the array, resetting the length back to zero.
            let arr = core::mem::replace(&mut self.arr, [(); N].map(|_| MaybeUninit::uninit()));
            self.len = 0;

            // SAFETY: we had len == N, so all elements in arr are initialized.
            Some(unsafe { arr.map(|v| v.assume_init()) })
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for ArrayBuilder<T, N> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: arr[..len] is initialized, so must be dropped.
            // First we create a pointer to this initialized slice, then drop
            // that slice in-place. The cast from *mut MaybeUninit<T> to *mut T
            // is always sound by the layout guarantees of MaybeUninit.
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
