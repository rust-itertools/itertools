#[macro_use]
extern crate itertools;

use itertools::assert_equal;
use std::ops::Add;

#[cfg(feature = "unstable")]
#[test]
fn icompr() {
    let log = "GET / 4096\nGET /home/ 16301\nPOST /home/ 49\nGET / 4096\n";
    let lines = log.lines();
    let rows = icompr!(line.split(|c: char| c.is_whitespace()) for line in lines);
    let ngets = icompr!(1 for mut row in rows if row.next() == Some("GET")).fold(0, Add::add);
    assert_eq!(ngets, 3);
}

#[cfg(feature = "unstable")]
#[test]
fn testcompr() {
    assert_equal(icompr!(x + 2 for x in 0..10), 2..12);
    assert_equal(icompr!(x for x in 0..3 if x != 1), vec![0, 2]);
    assert_equal(icompr!(x for (x, y) in (1..2).cycle().take(3).enumerate() if x != y),
                 vec![0, 2]);
}
