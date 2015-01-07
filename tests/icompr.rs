#[macro_use]
extern crate itertools;

use std::iter::AdditiveIterator;

#[test]
fn icompr() {
    let log = "GET / 4096\nGET /home/ 16301\nPOST /home/ 49\nGET / 4096\n";
    let lines = log.lines();
    let rows = icompr!(line.words() for line in lines);
    let ngets = icompr!(1i for mut row in rows if row.next() == Some("GET")).sum();
    assert_eq!(ngets, 3i);
}
