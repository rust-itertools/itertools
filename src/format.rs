use std::fmt;
use std::cell::RefCell;

/// Format all iterator elements lazily, separated by `sep`.
///
/// The format value can only be formatted once, after that the iterator is
/// exhausted.
///
/// See [`.format_with()`](../trait.Itertools.html#method.format_with) for more information.
pub struct FormatWith<'a, I, F> {
    sep: &'a str,
    /// FormatWith uses interior mutability because Display::fmt takes &self.
    inner: RefCell<Option<(I, F)>>,
}

/// Format all iterator elements lazily, separated by `sep`.
///
/// The format value can only be formatted once, after that the iterator is
/// exhausted.
///
/// See [`.format()`](../trait.Itertools.html#method.format)
/// for more information.
#[derive(Clone)]
pub struct Format<'a, I> {
    sep: &'a str,
    /// Format uses interior mutability because Display::fmt takes &self.
    inner: RefCell<Option<I>>,
}

/// See [`.format_with_bookends()`](../trait.Itertools.html#method.format_with_bookends)
pub struct FormatWithBookends<'a, I> {
    prefix: &'a str,
    sep: &'a str,
    suffix: &'a str,
    /// Format uses interior mutability because Display::fmt takes &self.
    inner: RefCell<Option<I>>,
}

pub fn new_format<'a, I, F>(iter: I, separator: &'a str, f: F) -> FormatWith<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result
{
    FormatWith {
        sep: separator,
        inner: RefCell::new(Some((iter, f))),
    }
}

pub fn new_format_default<'a, I>(iter: I, separator: &'a str) -> Format<'a, I>
    where I: Iterator,
{
    Format {
        sep: separator,
        inner: RefCell::new(Some(iter)),
    }
}

pub fn new_format_with_bookends<'a, I>(iter: I, prefix: &'a str, sep: &'a str, suffix: &'a str) -> FormatWithBookends<'a, I>
    where I: Iterator,
{
    FormatWithBookends {
        prefix: prefix,
        sep: sep,
        suffix: suffix,
        inner: RefCell::new(Some(iter))
    }
}

impl<'a, I, F> fmt::Display for FormatWith<'a, I, F>
    where I: Iterator,
          F: FnMut(I::Item, &mut FnMut(&fmt::Display) -> fmt::Result) -> fmt::Result
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (mut iter, mut format) = match self.inner.borrow_mut().take() {
            Some(t) => t,
            None => panic!("FormatWith: was already formatted once"),
        };

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

impl<'a, I> Format<'a, I>
    where I: Iterator,
{
    fn format<F>(&self, f: &mut fmt::Formatter, mut cb: F) -> fmt::Result
        where F: FnMut(&I::Item, &mut fmt::Formatter) -> fmt::Result,
    {
        let mut iter = match self.inner.borrow_mut().take() {
            Some(t) => t,
            None => panic!("Format: was already formatted once"),
        };

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
    ($fmt_struct:ident, $($fmt_trait:ident)*) => {
        $(
            impl<'a, I> fmt::$fmt_trait for $fmt_struct<'a, I>
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

impl_format!{Format,
  Display Debug UpperExp LowerExp UpperHex LowerHex Octal Binary Pointer}

impl<'a, I> FormatWithBookends<'a, I>
    where I: Iterator,
{
    fn format<F>(&self, f: &mut fmt::Formatter, mut cb: F) -> fmt::Result
        where F: FnMut(&I::Item, &mut fmt::Formatter) -> fmt::Result,
    {
        let mut iter = match self.inner.borrow_mut().take() {
            Some(t) => t,
            None => panic!("FormatWithBookends: was already formatted once"),
        };

        if let Some(fst) = iter.next() {
            try!(f.write_str(self.prefix));
            try!(cb(&fst, f));
            for elt in iter {
                if self.sep.len() > 0 {
                    try!(f.write_str(self.sep));
                }
                try!(cb(&elt, f));
            }
            try!(f.write_str(self.suffix));
        }
        Ok(())
    }
}

impl_format!{FormatWithBookends,
  Display Debug UpperExp LowerExp UpperHex LowerHex Octal Binary Pointer}
