use std::iter::ExactSizeIterator;

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
    iter: I,
    inner: Option<CircularArrayWindowsInner<I::Item, N>>,
}

#[derive(Debug, Clone)]
struct CircularArrayWindowsInner<T: Clone, const N: usize> {
    // `prefix` stores the first N items from the input iterator. If
    // the input contained fewer than `N` items, then it is filled
    // with clones of the previous items in a cycle.
    //
    // `prefix_pos` tracks the number of items that have been _used_
    // from `prefix`. It is `None` while the input iterator is still
    // running, and then counts up from 0 once the input runs out. (So
    // in the case where the input iterator is shorter than `N`, it
    // will begin counting up before `prefix` has even been populated
    // during setup.)
    //
    // If the input iterator contains `k` items, then our `k` output
    // windows of length `N` will cover `k+N-1` items in total. So we
    // have output enough windows precisely when `prefix_pos` reaches
    // `N-1`, whether or not we began incrementing `prefix_pos` during
    // initial setup.
    prefix: [T; N],
    prefix_pos: Option<usize>,

    // `ringbuf` stores the _most recent_ N items from the input
    // iterator, which were delivered in the most recent output
    // window. It is stored in the form of a ring buffer, with
    // `ringpos` identifying the element that logically comes first.
    ringbuf: [T; N],
    ringpos: usize,
}

impl<T: Clone, const N: usize> CircularArrayWindowsInner<T, N> {
    /// Make a new `CircularArrayWindowsInner`, in which `prefix`
    /// contains the item `first`, plus `N-1` more items from the
    /// provided iteraor (or recycle existing items if necessary).
    fn new(first: T, iter: &mut impl Iterator<Item = T>) -> Self {
        // To allow building up `prefix` incrementally, we make it in
        // the form of an array of `Option`. Once we've finished, and
        // all its elements are `Some`, we can map it through `unwrap`
        // to make the unconditional `[T; N]` that goes in the output
        // struct.
        let mut items = std::array::from_fn(|_| None);
        let mut prefix_pos = None;
        if N > 0 {
            // The first item stored is the one passed to us from our
            // caller.
            items[0] = Some(first);
        }
        for i in 1..N {
            // Populate the remaining slots in `items` from the input
            // iterator.
            let item = iter.next();
            if item.is_none() {
                // If the input iterator runs out early, populate the
                // rest of `items` by recycling from the beginning,
                // and set `prefix_pos` to indicate that we have
                // already consumed those items.
                for j in i..N {
                    items[j] = items[j - i].clone();
                }
                prefix_pos = Some(N - i);
                break;
            }
            items[i] = item;
        }
        let items = items.map(Option::unwrap);
        Self {
            prefix: items.clone(),
            prefix_pos,
            ringbuf: items,
            ringpos: 0,
        }
    }

    /// Read the next item in the logical input sequence (consisting
    /// of the contents of the input iterator followed by N-1 items
    /// recycling from the beginning). Add it to the ring buffer.
    fn read_item(&mut self, iter: &mut impl Iterator<Item = T>) -> bool {
        let item = if let Some(pos) = &mut self.prefix_pos {
            // The input iterator has already run out, so clone an
            // element from `prefix`, wrapping round to the start as
            // necessary.
            let item = self.prefix[*pos].clone();
            *pos += 1;
            item
        } else if let Some(item) = iter.next() {
            // Read from the input iterator.
            item
        } else if N > 1 {
            // The input iterator has run out right now, so clone the
            // first element of `prefix`, and set cyclepos to point to
            // the next one.
            self.prefix_pos = Some(1);
            self.prefix[0].clone()
        } else {
            // Special case if N=1: don't read an item at all if the
            // input iterator has run out.
            self.prefix_pos = Some(0);
            return false;
        };

        if N > 0 {
            self.ringbuf[self.ringpos] = item;
            self.ringpos = (self.ringpos + 1) % N;
        }
        true
    }

    /// Construct an array window to return, given the newly read item
    /// to go on the end of the output.
    fn make_window(&mut self) -> [T; N] {
        std::array::from_fn(|i| self.ringbuf[(i + self.ringpos) % N].clone())
    }
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

                    let mut inner = CircularArrayWindowsInner::new(first, &mut self.iter);
                    let window = inner.make_window();
                    self.inner = Some(inner);
                    Some(window)
                }
            },
            Some(inner) => {
                if let Some(pos) = inner.prefix_pos {
                    if pos + 1 >= N {
                        // The input iterator has run out, and we've
                        // emitted as many windows as we read items,
                        // so we've finished.
                        return None;
                    }
                }
                // Normal case. Fetch an item and return a window.
                if inner.read_item(&mut self.iter) {
                    Some(inner.make_window())
                } else {
                    None
                }
            }
        }
    }
}

// We return exactly one window per input item, so if the input
// iterator knows its length, then so do we.
impl<I, const N: usize> ExactSizeIterator for CircularArrayWindows<I, N>
where
    I: Iterator + Sized + ExactSizeIterator,
    I::Item: Clone,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

pub fn circular_array_windows<I, const N: usize>(iter: I) -> CircularArrayWindows<I, N>
where
    I: Iterator + Sized,
    I::Item: Clone,
{
    CircularArrayWindows { iter, inner: None }
}
