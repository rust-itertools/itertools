use core::mem::MaybeUninit;

/// Helper struct to build up an array element by element.
struct ArrayBuilder<T, const N: usize> {
    arr: [MaybeUninit<T>; N],
    i: usize
}

impl<T, const N: usize> ArrayBuilder<T, N> {
    pub fn new() -> Self {
        Self { arr: maybe_uninit::uninit_array(), i: 0 }
    }
    
    pub unsafe fn push_unchecked(&mut self, x: T) {
        debug_assert!(self.i < N);
        *self.arr.get_unchecked_mut(self.i) = MaybeUninit::new(x);
        self.i += 1;
    }
    
    pub fn take(mut self) -> Option<[T; N]> {
        if self.i == N {
            unsafe {
                // SAFETY: prevent double drop.
                self.i = 0;
                // SAFETY: [MaybeUninit<T>; N] and [T; N] have the same layout.
                let init_arr_ptr = &self.arr as *const _ as *const [T; N];
                Some(core::ptr::read(init_arr_ptr))
            }
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for ArrayBuilder<T, N> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: we only loop over the initialized portion.
            for el in &mut self.arr[..self.i] {
                maybe_uninit::assume_init_drop(el)
            }
        }
    }
}



/// Equivalent to `it.next_array()`.
pub fn next_array<I, T, const N: usize>(it: &mut I) -> Option<[T; N]>
where
    I: Iterator<Item = T>,
{
    let mut builder = ArrayBuilder::new();
    for el in it.take(N) {
        unsafe {
            // SAFETY: the take(N) guarantees we never go out of bounds.
            builder.push_unchecked(el);
        }
    }
    builder.take()
}



/// Replacements for unstable core methods, copied from stdlib.
mod maybe_uninit {
    use core::mem::MaybeUninit;

    pub fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
        // SAFETY: an uninitialized `[MaybeUninit<_>; N]` is valid.
        unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
    }

    pub unsafe fn assume_init_drop<T>(u: &mut MaybeUninit<T>) {
        // SAFETY: the caller must guarantee that `self` is initialized and
        // satisfies all invariants of `T`.
        // Dropping the value in place is safe if that is the case.
        core::ptr::drop_in_place(u.as_mut_ptr())
    }
}
