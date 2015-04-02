#[macro_use]
extern crate itertools;

use std::ops::Add;

#[test]
fn icompr() {
    let log = "GET / 4096\nGET /home/ 16301\nPOST /home/ 49\nGET / 4096\n";
    let lines = log.lines();
    let rows = icompr!(line.split(|c: char| c.is_whitespace()), line, lines);
    let ngets = icompr!(1, mut row, rows, row.next() == Some("GET")).fold(0, Add::add);
    assert_eq!(ngets, 3);
}
