use crate::Itertools;
use std::iter::Fuse;

/// An iterator over all contiguous windows of the input iterator,
/// producing arrays of a specific size.
///
/// See [`.array_windows()`](crate::Itertools::array_windows) for more
/// information.
#[derive(Debug, Clone)]
pub struct ArrayWindows<I, const N: usize>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    iter: Fuse<I>,
    inner: Option<ArrayWindowsInner<I::Item, N>>,
}

#[derive(Debug, Clone)]
struct ArrayWindowsInner<T: Clone, const N: usize> {
    // `window` stores the `N` items delivered in the most
    // recent output window. It is stored in the form of a ring
    // buffer, with `window_start` identifying the element
    // that logically comes first.
    window: [T; N],
    window_start: usize,
}

impl<T: Clone, const N: usize> ArrayWindowsInner<T, N> {
    /// Replace the least recent item in `window` with a new
    /// item.
    fn add_to_buffer(&mut self, item: T) {
        if N > 0 {
            self.window[self.window_start] = item;
            self.window_start = (self.window_start + 1) % N;
        }
    }

    /// Construct an array window to return.
    fn make_window(&self) -> [T; N] {
        std::array::from_fn(|i| self.window[(i + self.window_start) % N].clone())
    }
}

impl<I, const N: usize> Iterator for ArrayWindows<I, N>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<[I::Item; N]> {
        match &mut self.inner {
            // Initialisation code, when next() is called for the first time
            None => match self.iter.next_array() {
                None => {
                    // The input iterator was completely empty
                    None
                }
                Some(buf) => {
                    let inner = ArrayWindowsInner {
                        window: buf.clone(),
                        window_start: 0,
                    };
                    let window = inner.make_window();
                    self.inner = Some(inner);
                    Some(window)
                }
            },
            Some(inner) => match self.iter.next() {
                Some(item) => {
                    inner.add_to_buffer(item);
                    Some(inner.make_window())
                }
                None => None,
            },
        }
    }
}

pub fn array_windows<I, const N: usize>(iter: I) -> ArrayWindows<I, N>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    ArrayWindows {
        iter: iter.fuse(),
        inner: None,
    }
}

/// An iterator over all windows, wrapping back to the first elements when the
/// window would otherwise exceed the length of the iterator, producing arrays
/// of a specific size.
///
/// See [`.circular_array_windows()`](crate::Itertools::circular_array_windows)
/// for more information.
#[derive(Debug, Clone)]
pub struct CircularArrayWindows<I, const N: usize>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    iter: Fuse<I>,
    inner: Option<CircularArrayWindowsInner<I::Item, N>>,
}

#[derive(Debug, Clone)]
struct CircularArrayWindowsInner<T: Clone, const N: usize> {
    // `prefix` stores the first `N` items output from this iterator.
    // If the input contained fewer than `N` items, then it is filled
    // with clones of the previous items in a cycle.
    //
    // `prefix_pos` tracks the number of items that have been _used_
    // from `prefix`. It begins counting up from 0 once the input runs
    // out. (So in the case where the input iterator is shorter than
    // `N`, it will begin counting up before `prefix` has even been
    // populated during setup.)
    prefix: [T; N],
    prefix_pos: usize,

    // For delivering the output arrays, we reuse `ArrayWindowsInner`
    // unchanged.
    arraywin: ArrayWindowsInner<T, N>,
}

impl<I, const N: usize> Iterator for CircularArrayWindows<I, N>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<[I::Item; N]> {
        match &mut self.inner {
            // Initialisation code, when next() is called for the first time
            None => match self.iter.next() {
                None => {
                    // The input iterator was completely empty
                    None
                }
                Some(first) => {
                    // We have at least one item, so we can definitely
                    // populate `prefix` (even if we have to make N
                    // copies of this element).

                    // Construct [Option<T>; N] and convert to [T; N]
                    // once it's full. TODO: can this be improved?
                    let mut items = std::array::from_fn(|_| None);
                    let mut prefix_pos = 0;
                    if N > 0 {
                        // The first item stored is the one passed to
                        // us from our caller.
                        items[0] = Some(first);
                    }
                    for i in 1..N {
                        // Populate the remaining slots in `items`
                        // from the input iterator.
                        items[i] = self.iter.next();
                        if items[i].is_none() {
                            // If the input iterator runs out early,
                            // populate the rest of `items` by
                            // recycling from the beginning, and set
                            // `prefix_pos` to indicate that we have
                            // already consumed those items.
                            for j in i..N {
                                items[j] = items[j - i].clone();
                            }
                            prefix_pos = N - i;
                            break;
                        }
                    }
                    let items = items.map(Option::unwrap);

                    let inner = CircularArrayWindowsInner {
                        prefix: items.clone(),
                        prefix_pos,
                        arraywin: ArrayWindowsInner {
                            window: items,
                            window_start: 0,
                        },
                    };

                    let window = inner.arraywin.make_window();
                    self.inner = Some(inner);
                    Some(window)
                }
            },
            Some(inner) => {
                // Normal case. Read the next item in the logical
                // input sequence (consisting of the contents of the
                // input iterator followed by N-1 items recycling from
                // the beginning), and add it to the ring buffer.
                let item = if let Some(item) = self.iter.next() {
                    // Read from the input iterator.
                    item
                } else if N == 0 {
                    return None;
                } else {
                    assert!(N == 0 || inner.prefix_pos < N);
                    if inner.prefix_pos + 1 == N {
                        // The input iterator has run out, and we've
                        // emitted as many windows as we read items,
                        // so we've finished.
                        return None;
                    }
                    let item = inner.prefix[inner.prefix_pos].clone();
                    inner.prefix_pos += 1;
                    item
                };

                if N > 0 {
                    inner.arraywin.add_to_buffer(item);
                }
                Some(inner.arraywin.make_window())
            }
        }
    }
}

pub fn circular_array_windows<I, const N: usize>(iter: I) -> CircularArrayWindows<I, N>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    CircularArrayWindows {
        iter: iter.fuse(),
        inner: None,
    }
}
