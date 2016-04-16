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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ref mut iter, ref mut format) = *self.inner.borrow_mut();

        if let Some(fst) = iter.next() {
            try!(format(fst, &mut |disp: &fmt::Display| disp.fmt(f)));
            for elt in iter {
                if self.sep.len() > 0 {

                    try!(f.write_str(self.sep));
                }
                try!(format(elt, &mut |disp: &fmt::Display| disp.fmt(f)));
            }
        }
        Ok(())
    }
}

impl<'a, I> FormatDefault<'a, I>
    where I: Iterator,
{
    fn format<F>(&self, f: &mut fmt::Formatter, mut cb: F) -> fmt::Result
        where F: FnMut(&I::Item, &mut fmt::Formatter) -> fmt::Result,
    {
        let iter = &mut *self.inner.borrow_mut();

        if let Some(fst) = iter.next() {
            try!(cb(&fst, f));
            for elt in iter {
                if self.sep.len() > 0 {
                    try!(f.write_str(self.sep));
                }
                try!(cb(&elt, f));
            }
        }
        Ok(())
    }
}

macro_rules! impl_format {
    ($($fmt_trait:ident)*) => {
        $(
            impl<'a, I> fmt::$fmt_trait for FormatDefault<'a, I>
                where I: Iterator,
                      I::Item: fmt::$fmt_trait,
            {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.format(f, fmt::$fmt_trait::fmt)
                }
            }
        )*
    }
}

impl_format!{Display Debug
             UpperExp LowerExp UpperHex LowerHex Octal Binary Pointer}
