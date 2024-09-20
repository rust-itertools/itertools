use core::mem::{self, MaybeUninit};

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

impl<T, const N: usize> AsMut<[T]> for ArrayBuilder<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        let valid = &mut self.arr[..self.len];
        // SAFETY: By invariant on `self.arr`, the elements of `self.arr` at
        // indices `0..self.len` are in a valid state. Since `valid` references
        // only these elements, the safety precondition of
        // `slice_assume_init_mut` is satisfied.
        unsafe { slice_assume_init_mut(valid) }
    }
}

impl<T, const N: usize> Drop for ArrayBuilder<T, N> {
    // We provide a non-trivial `Drop` impl, because the trivial impl would be a
    // no-op; `MaybeUninit<T>` has no innate awareness of its own validity, and
    // so it can only forget its contents. By leveraging the safety invariant of
    // `self.arr`, we do know which elements of `self.arr` are valid, and can
    // selectively run their destructors.
    fn drop(&mut self) {
        let valid = self.as_mut();
        // SAFETY: TODO
        unsafe { core::ptr::drop_in_place(valid) }
    }
}

/// Assuming all the elements are initialized, get a mutable slice to them.
///
/// # Safety
///
/// The caller guarantees that the elements `T` referenced by `slice` are in a
/// valid state.
unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: Casting `&mut [MaybeUninit<T>]` to `&mut [T]` is sound, because
    // `MaybeUninit<T>` is guaranteed to have the same size, alignment and ABI
    // as `T`, and because the caller has guaranteed that `slice` is in the
    // valid state.
    unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
}

/// Equivalent to `it.next_array()`.
pub fn next_array<I, const N: usize>(it: &mut I) -> Option<[I::Item; N]>
where
    I: Iterator,
{
    let mut builder = ArrayBuilder::new();
    for _ in 0..N {
        builder.push(it.next()?);
    }
    builder.take()
}
