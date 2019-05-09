/// Copy all iterator's elements into one slice and return number of elements.


/// Error variant if slice is exceeded
pub enum Error {
    /// End of slice reached
    EndOfSlice
}

/// Trait copying all iterator's elements into one slice, returning number of elements
///
/// The method will return Error::EndOfSlice if slice is exceeded
///
/// # Examples
/// ```
///    use itertools::CopyIntoSlice;
///
///    let req_method_get = b"GET ";
///    let req_method_head = b"HEAD ";
///    let uri = b"/infotext.html HTTP/1.1";
///
///    let mut request = [0 as u8; 128];
///
///    // chaining both iterators and copying bytewise into the buffer
///    match  req_method_get.iter()
///        .chain(uri.iter())
///        .copy_into_slice(request.as_mut())
///        {
///            Ok(nwritten)
///            => println!("{:?}", std::str::from_utf8(&request[0..nwritten])),
///            _ => panic!(),
///        };
/// ```
pub trait CopyIntoSlice<'t, It: 't>: Iterator<Item=&'t It> {
    /// Trait-method copying all iterator's elements into one slice, returning number of elements.
    ///
    /// Returns number of placed elements, otherwise Error if slice is exceeded.
    ///
    /// # Examples
    /// ```
    ///    use itertools::CopyIntoSlice;
    ///
    ///    let req_method_get = b"GET ";
    ///    let req_method_head = b"HEAD ";
    ///    let uri = b"/infotext.html HTTP/1.1";
    ///
    ///    let mut request = [0 as u8; 128];
    ///
    ///    // chaining both iterators and copying bytewise into the buffer
    ///    match  req_method_get.iter()
    ///        .chain(uri.iter())
    ///        .copy_into_slice(request.as_mut())
    ///        {
    ///            Ok(nwritten)
    ///            => println!("{:?}", std::str::from_utf8(&request[0..nwritten])),
    ///            _ => panic!(),
    ///        };
    /// ```
    fn copy_into_slice(self, slice: &mut [It]) -> Result<usize, Error>
        where
            It: Clone,
            Self: Sized,
    {
        let slice_len = slice.len();

        let mut nwritten = 0;

        for (idx, item) in self.enumerate() {
            if idx < slice_len {
                slice[idx] = item.clone();
                nwritten += 1;
            } else {
                return Err(Error::EndOfSlice);
            }
        }

        Ok(nwritten)
    }
}

/// Implementing the trait CopyIntoSlice
impl<'t, It: 't, I: Iterator<Item=&'t It> > CopyIntoSlice<'t, It> for I {}
