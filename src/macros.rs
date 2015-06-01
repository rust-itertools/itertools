/// `icompr` as in “iterator comprehension” allows creating a
/// mapped iterator with simple syntax, similar to set builder notation,
/// and directly inspired by Python. Supports an optional filter clause.
///
/// Syntax in form (1):
///
///  `icompr!(<expression> for <pattern> in <iterator>)`
///
/// or (2):
///
///  `icompr!(<expression> for <pattern> in <iterator> if <expression>)`
///
/// or (3):
///
///  `icompr!(<expression> for let <pattern> in <iterator> [ if <expression> ])`
///
/// Each element from the `<iterator>` expression is pattern matched
/// with the `<pattern>`, and the bound names are used to express the
/// mapped-to value.
///
/// Form (3) is a `for let` loop, which skips elements that don't match
/// a *refutable* pattern.
///
/// Iterator element type is the type of `<expression>`
///
/// ```
/// #[macro_use]
/// extern crate itertools;
///
/// # fn main() {
/// let squares = icompr!(x * x for x in 1..10);
/// itertools::assert_equal(squares, vec![1, 4, 9, 16, 25, 36, 49, 64, 81]);
///
/// let odds = icompr!(y for y in 0..6 if y % 2 == 1);
/// itertools::assert_equal(odds, vec![1, 3, 5]);
///
/// let numbers = icompr!(x for let Some(x) in vec![Some(1), None]);
/// itertools::assert_equal(numbers, vec![1]);
/// # }
/// ```
///
/// Note: This macro is relatively convoluted, so it may give harder to
/// understand error messages than usual when encountering missing arguments
/// or syntax errors.
#[macro_export]
macro_rules! icompr {
    ($($t:tt)+) => { itertools_icompr_internal!(start $($t)+); }
}

// Split a stream of tt's into before and after `for`
#[doc(hidden)]
#[macro_export]
macro_rules! itertools_split_for {
    ($sep: tt ($m: ident $($args:tt)*) [$($stack:tt)*] $x: tt for $($rest: tt)*) => {
        $m ! ($($args)* $($stack)* $x $sep $($rest)*)
    };
    ($sep: tt ($m:ident $($args:tt)*) [$($stack:tt)*] $x: tt $($rest: tt)*) => {
        itertools_split_for!($sep ($m $($args)*) [$($stack)* $x] $($rest)*);
    };
}

// Split a stream of tt's into before and after `if`
#[doc(hidden)]
#[macro_export]
macro_rules! itertools_split_if {
    ($sep: tt ($m: ident $($args:tt)*) [$($stack:tt)*] $x: tt) => {
        $m ! ($($args)* $($stack)* $x $sep)
    };
    ($sep: tt ($m: ident $($args:tt)*) [$($stack:tt)*] $x: tt if $($rest: tt)*) => {
        $m ! ($($args)* $($stack)* $x $sep $($rest)*)
    };
    ($sep: tt ($m:ident $($args:tt)*) [$($stack:tt)*] $x: tt $($rest: tt)*) => {
        itertools_split_if!($sep ($m $($args)*) [$($stack)* $x] $($rest)*);
    };
}

// Remove optional `let`
#[doc(hidden)]
#[macro_export]
macro_rules! itertools_strip_let {
    (($m: ident $($args:tt)*) let $($rest:tt)+) => {
        $m ! ($($args)* yes $($rest)+)
    };
    (($m: ident $($args:tt)*) $($rest:tt)+) => {
        $m ! ($($args)* no $($rest)+)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! itertools_icompr_internal {
    (start $($t:tt)+) => {
        itertools_split_for!(=> (itertools_icompr_internal match_let) [] $($t)+);
    };
    (match_let $e:expr => $($rest:tt)+) => {
        itertools_strip_let!((itertools_icompr_internal match_if $e =>) $($rest)+);
    };
    (match_if $e:expr => $yesno:ident $p:pat in $($iter:tt)+) => {
        itertools_split_if!(=> (itertools_icompr_internal build $yesno $e => $p =>) [] $($iter)+);
    };
    (build no $e:expr => $p:pat => $iter:expr =>) => {
        ::std::iter::IntoIterator::into_iter($iter).map(|$p| $e)
    };
    (build no $e:expr => $p:pat => $iter:expr => $pred:expr) => {
        ::std::iter::IntoIterator::into_iter($iter)
            .filter_map(|$p| if $pred { Some($e) } else { None })
    };
    (build yes $e:expr => $p:pat => $iter:expr =>) => {
        ::std::iter::IntoIterator::into_iter($iter).filter_map(|elt|
            match elt {
                $p => Some($e),
                _ => None,
            }
        )
    };
    (build yes $e:expr => $p:pat => $iter:expr => $pred:expr) => {
        ::std::iter::IntoIterator::into_iter($iter).filter_map(|elt|
            match elt {
                $p => if $pred { Some($e) } else { None },
                _ => None,
            }
        )
    };
}

