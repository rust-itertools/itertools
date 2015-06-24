use std::fmt;
use std::cell::RefCell;

/// Format all iterator elements lazily, separated by `sep`.
///
/// See [`.format()`](trait.Itertools.html#method.format) for more information.
pub struct Format<'a, I, F> {
    sep: &'a str,
    /// Format uses interior mutability because Display::fmt takes &self.
    inner: RefCell<(I, F)>,
}

pub fn new_format<'a, I, F>(iter: I, separator: &'a str, f: F) -> Format<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result,
{
    Format{sep: separator, inner: RefCell::new((iter, f))}
}

impl<'a, I, F> fmt::Display for Format<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut cb = &mut |disp: &fmt::Display| write!(fmt, "{}", disp);
        let mut inner = self.inner.borrow_mut();
        let mut trailing = false;
        while let Some(elt) = inner.0.next() {
            if trailing && self.sep.len() > 0 {
                try!(cb(&self.sep));
            }
            try!((&mut inner.1)(elt, cb));
            trailing = true;
        }
        Ok(())
    }
}

impl<'a, I, F> fmt::Debug for Format<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

