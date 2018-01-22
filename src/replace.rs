pub fn replace<C, I, R>(iter: I, cond: C, replacement: R) -> Replace<C, I, R>
    where C: FnMut(&I::Item) -> bool,
          I: Iterator,
          R: Iterator<Item=I::Item> + Clone
{
    Replace { cond: cond, iter: iter, with_orig: replacement, with: None }
}

/// An iterator adaptor that replaces occurrences of an item with a different sequence of items.
///
/// It is returned by [`Itertools::replace`].
///
/// [`Itertools::replace`]: ../trait.Itertools.html#method.replace
pub struct Replace<C, I, R>
    where C: FnMut(&I::Item) -> bool,
          I: Iterator,
          R: Iterator<Item=I::Item> + Clone
{
    cond: C,
    iter: I,
    with: Option<R>,
    with_orig: R,
}

impl<C, I, R> Iterator for Replace<C, I, R>
    where C: FnMut(&I::Item) -> bool,
          I: Iterator,
          R: Iterator<Item=I::Item> + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        loop {
            self.with = match self.with {
                Some(ref mut replacement) => {
                    match replacement.next() {
                        Some(item) => {
                            // emit replacement item
                            return Some(item);
                        }
                        None => {
                            // continue with original iterator
                            None
                        }
                    }
                }
                None => {
                    match self.iter.next() {
                        None => return None,
                        Some(item) => if (self.cond)(&item) {
                            // emit the replacement iterator once
                            Some(self.with_orig.clone())
                        } else {
                            return Some(item);
                        }
                    }
                }
            }
        }
    }
}
