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
        // SAFETY: The safety invariant of `arr` trivially holds for `len = 0`.
        Self {
            arr: [(); N].map(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }

    /// Pushes `value` onto the end of the array.
    ///
    /// # Panics
    ///
    /// This panics if `self.len >= N` or if `self.len == usize::MAX`.
    pub fn push(&mut self, value: T) {
        // PANICS: This will panic if `self.len >= N`.
        // SAFETY: The safety invariant of `self.arr` applies to elements at
        // indices `0..self.len` — not to the element at `self.len`. Writing to
        // the element at index `self.len` therefore does not violate the safety
        // invariant of `self.arr`. Even if this line panics, we have not
        // created any intermediate invalid state.
        self.arr[self.len] = MaybeUninit::new(value);
        // PANICS: This will panic if `self.len == usize::MAX`.
        // SAFETY: By invariant on `self.arr`, all elements at indicies
        // `0..self.len` are valid. Due to the above write, the element at
        // `self.len` is now also valid. Consequently, all elements at indicies
        // `0..(self.len + 1)` are valid, and `self.len` can be safely
        // incremented without violating `self.arr`'s invariant. It is fine if
        // this increment panics, as we have not created any intermediate
        // invalid state.
        self.len = match self.len.checked_add(1) {
            Some(sum) => sum,
            None => panic!("`self.len == usize::MAX`; cannot increment `len`"),
        };
    }

    /// Consumes the elements in the `ArrayBuilder` and returns them as an array
    /// `[T; N]`.
    ///
    /// If `self.len() < N`, this returns `None`.
    pub fn take(&mut self) -> Option<[T; N]> {
        if self.len == N {
            // SAFETY: Decreasing the value of `self.len` cannot violate the
            // safety invariant on `self.arr`.
            self.len = 0;

            // SAFETY: Since `self.len` is 0, `self.arr` may safely contain
            // uninitialized elements.
            let arr = mem::replace(&mut self.arr, [(); N].map(|_| MaybeUninit::uninit()));

            Some(arr.map(|v| {
                // SAFETY: We know that all elements of `arr` are valid because
                // we checked that `len == N`.
                unsafe { v.assume_init() }
            }))
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for ArrayBuilder<T, N> {
    // We provide a non-trivial `Drop` impl, because the trivial impl would be a
    // no-op; `MaybeUninit<T>` has no innate awareness of its own validity, and
    // so it can only forget its contents. By leveraging the safety invariant of
    // `self.arr`, we do know which elements of `self.arr` are valid, and can
    // selectively run their destructors.
    fn drop(&mut self) {
        // Select the valid elements of `self.arr`.
        //
        // LEMMA 1: The elements of `valid` reference the valid elements of
        // `self.arr`.
        //
        // PROOF: `slice::split_at_mut(mid)` produces a pair of slices, the
        // first of which contains the elements at the indices `0..mid`. By
        // invariant on `self.arr`, the elements of `self.arr` at indicies
        // `0..self.len` are valid. Assuming that `slice::split_at_mut` is
        // correctly implemented, the slice `valid` will only reference the
        // valid elements of `self.arr`.
        let (valid, _) = self.arr.split_at_mut(self.len);

        // Cast `valid` from `&[MaybeUninit<T>]` to `&[T]`
        //
        // `align_to_mut` guarantees that the length of the casted slice will be
        // as long as possible within the constraints of the source and
        // destination element types' alignments and sizes. Since
        // `MaybeUninit<T>` and `T` have identical alignments and sizes [1], all
        // elements will be casted and the prefix and suffix components of the
        // return value will be empty and `valid` will contain all of the
        // elements that it did prior to the cast.
        //
        // SAFETY: It is sound to cast a `MaybeUninit<T>` that contains a valid
        // `T` to a `T`. A `MaybeUninit<T>` is guaranteed to have the same size,
        // alignment, and ABI as `T` [1], and by LEMMA 1, `valid` consists only
        // of `MaybeUninit<T>` in the initialized state.
        //
        // [1]: https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#layout-1
        let (_, valid, _): (_, &mut [T], _) = unsafe { valid.align_to_mut::<T>() };

        // LEMMA 2: `valid_ptr` has exactly the same safety invariants as
        // `valid`.
        //
        // PROOF: We assume that `slice::as_mut_ptr` correctly produces a raw
        // `mut` slice pointing to the same elements as its receiver. Such a
        // pointer will be valid for both reads and writes, be properly aligned,
        // and be non-null. By `mem::forget`ting `valid`, we additionally ensure
        // that `valid_ptr` is the *only* pointer to its referent.
        let valid_ptr = {
            let ptr = valid.as_mut_ptr();
            // Move `valid` out of the surrounding scope and immediately drop
            // it. `ptr` is now the only pointer to `valid`'s referent.
            drop(valid);
            ptr
        };

        // Run the destructors of `valid_ptr`'s elements.
        //
        // SAFETY:
        // - `valid_ptr`, by LEMMA 2, is valid for both reads and writes
        // - `valid_ptr`, by LEMMA 2, is properly aligned
        // - `valid_ptr`, by LEMMA 2, is non-null
        // - `valid_ptr`, by LEMMA 2, is valid for dropping, because it is data
        //   owned by the `ArrayBuilder` and we place no additional drop-related
        //   invariants on it
        // - `valid_ptr`, by LEMMA 2, is the only pointer to its referent, and
        //   therefore its referent cannot be concurrently accessed during the
        //   execution of `ptr::drop_in_place`.
        // - The referent of `valid_ptr`, which may not be `Copy` is not re-used
        //   after the invocation of `ptr::drop_in_place`.
        unsafe {
            ptr::drop_in_place(valid_ptr);
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
