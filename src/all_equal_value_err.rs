#[cfg(doc)]
use crate::Itertools;
#[cfg(feature = "use_std")]
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Value returned for the error case of [`Itertools::all_equal_value`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllEqualValueError<Item>(pub Option<[Item; 2]>);

impl<Item> Display for AllEqualValueError<Item> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.0 {
            None => {
                write!(
                    f,
                    "got zero elements when all elements were expected to be equal"
                )
            }
            Some([_, _]) => {
                write!(
                    f,
                    "got different elements when all elements were expected to be equal"
                )
            }
        }
    }
}

#[cfg(feature = "use_std")]
impl<Item> Error for AllEqualValueError<Item> where Item: Debug {}
