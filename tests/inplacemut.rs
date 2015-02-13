//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

#![feature(test)]
extern crate itertools;

extern crate test;

use itertools::CombineError::*;
use itertools::Combine::*;
use itertools::InplaceMappable;

use std::cmp::min;
use Thing::*;

#[derive(PartialEq, Debug)]
enum Thing { A, B, AandB }

#[test]
fn out_of_range() {
    let v = vec![A, B, A, A, A, B, B, A];
    let v = v.combine(|arr|
        match &arr[..min(2, arr.len())] {
            [..] => Keep(99)
        }
    ).unwrap_err();
    assert_eq!(v, OutOfRange);
}


#[test]
fn zero_forward() {
    let v = vec![A, B, A, A, A, B, B, A];
    let v = v.combine(|arr|
        match &arr[..min(2, arr.len())] {
            [..] => Keep(0)
        }
    ).unwrap_err();
    assert_eq!(v, ZeroForward);
}

#[test]
fn zero_forward2() {
    let v = vec![A, B, A, A, A, B, B, A];
    let v = v.combine(|arr|
        match &arr[..min(2, arr.len())] {
            [..] => InsertAndDrop(AandB, 0)
        }
    ).unwrap_err();
    assert_eq!(v, ZeroForward);
}
