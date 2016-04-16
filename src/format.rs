use std::fmt;
use std::cell::RefCell;

/// Format all iterator elements lazily, separated by `sep`.
///
/// The format value can only be formatted once, after that the iterator is
/// exhausted.
///
/// See [`.format()`](trait.Itertools.html#method.format) for more information.
pub struct Format<'a, I, F> {
    sep: &'a str,
    /// Format uses interior mutability because Display::fmt takes &self.
    inner: RefCell<(I, F)>,
}

/// Format all iterator elements lazily, separated by `sep`.
///
/// The format value can only be formatted once, after that the iterator is
/// exhausted.
///
/// See [`.format_default()`](trait.Itertools.html#method.format_default)
/// for more information.
#[derive(Clone)]
pub struct FormatDefault<'a, I> {
    sep: &'a str,
    /// Format uses interior mutability because Display::fmt takes &self.
    inner: RefCell<I>,
}

pub fn new_format<'a, I, F>(iter: I, separator: &'a str, f: F) -> Format<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result
{
    Format {
        sep: separator,
        inner: RefCell::new((iter, f)),
    }
}

pub fn new_format_default<'a, I>(iter: I, separator: &'a str) -> FormatDefault<'a, I>
    where I: Iterator,
{
    FormatDefault {
        sep: separator,
        inner: RefCell::new(iter),
    }
}

impl<'a, I, F> fmt::Display for Format<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut cb = &mut |disp: &fmt::Display| write!(fmt, "{}", disp);
        let (ref mut iter, ref mut format) = *self.inner.borrow_mut();

        if let Some(fst) = iter.next() {
            try!(format(fst, cb));
            for elt in iter {
                if self.sep.len() > 0 {
                    try!(cb(&self.sep));
                }
                try!(format(elt, cb));
            }
        }
        Ok(())
    }
}

impl<'a, I> fmt::Display for FormatDefault<'a, I>
    where I: Iterator,
          I::Item: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = &mut *self.inner.borrow_mut();

        if let Some(fst) = iter.next() {
            try!(write!(f, "{}", fst));
            for elt in iter {
                if self.sep.len() > 0 {
                    try!(f.write_str(self.sep));
                }
                try!(write!(f, "{}", elt));
            }
        }
        Ok(())
    }
}

impl<'a, I> fmt::Debug for FormatDefault<'a, I>
    where I: Iterator,
          I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = &mut *self.inner.borrow_mut();

        if let Some(fst) = iter.next() {
            try!(write!(f, "{:?}", fst));
            for elt in iter {
                if self.sep.len() > 0 {
                    try!(f.write_str(self.sep));
                }
                try!(write!(f, "{:?}", elt));
            }
        }
        Ok(())
    }
}
