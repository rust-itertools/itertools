/// Value returned for the error case of `Itertools::all_equal_value()`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllEqualValueError<Item>(pub Option<[Item; 2]>);
