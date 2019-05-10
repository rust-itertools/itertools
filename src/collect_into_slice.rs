/// Copy all iterator's elements into one slice and return number of elements.


/// Error variant if slice is exceeded
pub enum Error {
    /// End of slice reached
    EndOfSlice
}

/// Trait collectin all iterator elements into one slice, returning number of elements
///
/// The method will return Error::EndOfSlice if slice is exceeded
///
/// # Examples
/// ```
///    use itertools::CollectIntoSlice;
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
///        .collect_into_slice(request.as_mut())
///        {
///            Ok(nwritten)
///            => println!("{:?}", request[0..nwritten].as_ref()),
///            _ => panic!(),
///        };
/// ```
pub trait CollectIntoSlice<'t, It: 't>: Iterator<Item=&'t It> {
    /// Trait-method copying all iterator's elements into one slice, returning number of elements.
    ///
    /// Returns number of placed elements, otherwise Error if slice is exceeded.
    ///
    /// # Examples
    /// ```
    ///    use itertools::CollectIntoSlice;
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
    ///        .collect_into_slice(request.as_mut())
    ///        {
    ///            Ok(nwritten)
    ///            => println!("{:?}", request[0..nwritten].as_ref()),
    ///            _ => panic!(),
    ///        };
    /// ```
    fn collect_into_slice(self, slice: &mut [It]) -> Result<usize, Error>
        where
            It: Clone,
            Self: Sized,
    {
        let mut nwritten = 0;
        let mut iter = self.fuse();
        iter.by_ref()
            .zip(slice)
            .for_each(| (item, slot)| {*slot = item.clone(); nwritten+=1; } );

        // conditional return value
        match iter.next() {
            None => Ok(nwritten),
            _ => Err(Error::EndOfSlice),
        }
    }
}

/// Implementing the trait CollectIntoSlice
impl<'t, It: 't, I: Iterator<Item=&'t It> > CollectIntoSlice<'t, It> for I {}
