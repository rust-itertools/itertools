/// `MinMaxResult` is an enum returned by `minmax`. See `Itertools::minmax()` for
/// more detail.
#[derive(PartialEq, Debug)]
pub enum MinMaxResult<T> {
    /// Empty iterator
    NoElements,

    /// Iterator with one element, so the minimum and maximum are the same
    OneElement(T),

    /// More than one element in the iterator, the first element is not larger
    /// than the second
    MinMax(T, T)
}

impl<T: Clone> MinMaxResult<T> {
    /// `into_option` creates an `Option` of type `(T, T)`. The returned `Option`
    /// has variant `None` if and only if the `MinMaxResult` has variant
    /// `NoElements`. Otherwise variant `Some(x, y)` is returned where `x <= y`.
    /// If `MinMaxResult` has variant `OneElement(x)`, performing this operation
    /// will make one clone of `x`.
    ///
    /// # Examples
    ///
    /// ```
    /// use itertools::MinMaxResult::{self, NoElements, OneElement, MinMax};
    ///
    /// let r: MinMaxResult<i32> = NoElements;
    /// assert_eq!(r.into_option(), None);
    ///
    /// let r = OneElement(1);
    /// assert_eq!(r.into_option(), Some((1, 1)));
    ///
    /// let r = MinMax(1, 2);
    /// assert_eq!(r.into_option(), Some((1, 2)));
    /// ```
    pub fn into_option(self) -> Option<(T,T)> {
        match self {
            MinMaxResult::NoElements => None,
            MinMaxResult::OneElement(x) => Some((x.clone(), x)),
            MinMaxResult::MinMax(x, y) => Some((x, y))
        }
    }
}
