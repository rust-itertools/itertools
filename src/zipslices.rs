use std::cmp;

#[derive(Clone, Debug)]
pub struct ZipSlices<'a, T: 'a, U :'a>
{
    t: &'a [T],
    u: &'a [U],
    len: usize,
    index: usize,
}

impl<'a, T, U> ZipSlices<'a, T, U> {
    #[inline(always)]
    pub fn new(t: &'a [T], u: &'a [U]) -> Self {
        let minl = cmp::min(t.len(), u.len());
        ZipSlices {
            t: t,
            u: u,
            len: minl,
            index: 0,
        }
    }
}

impl<'a, T, U> Iterator for ZipSlices<'a, T, U> {
    type Item = (&'a T, &'a U);

    #[inline(always)]
    fn next(&mut self) -> Option<(&'a T, &'a U)> {
        unsafe {
            if self.index >= self.len {
                None
            } else {
                let i = self.index;
                self.index += 1;
                Some((
                    self.t.get_unchecked(i),
                    self.u.get_unchecked(i)))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut len = self.len - self.index;
        (len, Some(len))
    }
}

impl<'a, T, U> DoubleEndedIterator for ZipSlices<'a, T, U> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<(&'a T, &'a U)> {
        unsafe {
            if self.index == self.len {
                None
            } else {
                self.len -= 1;
                let i = self.len;
                Some((
                    self.t.get_unchecked(i),
                    self.u.get_unchecked(i)))
            }
        }
    }
}

impl<'a, T, U> ExactSizeIterator for ZipSlices<'a, T, U> { }

