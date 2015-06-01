//#![feature(trace_macros)]
#[macro_use]
extern crate itertools;

use itertools::assert_equal;

#[cfg(feature = "unstable")]
#[test]
fn icompr() {
    //trace_macros!(true);
    let log = "GET / 4096\nGET /home/ 16301\nPOST /home/ 49\nGET / 4096\n";
    let lines = log.lines();
    let rows = icompr!(line.split(|c: char| c.is_whitespace()) for line in lines);
    let firsts = icompr!(row.next() for mut row in rows);
    let ngets = icompr!(1 for let Some("GET") in firsts).count();
    assert_eq!(ngets, 3);

    let ngets = log.lines()
                   .filter(|line| line.starts_with("GET"))
                   .count();
    assert_eq!(ngets, 3);
}

#[cfg(feature = "unstable")]
#[test]
fn testcompr() {
    //trace_macros!(true);
    assert_equal(icompr!(x + 2 for x in 0..10), 2..12);
    assert_equal(icompr!(x for x in 0..3 if x != 1), vec![0, 2]);
    assert_equal(icompr!(x for (x, y) in (1..2).cycle().take(3).enumerate() if x != y),
                 vec![0, 2]);

    // for let !
    assert_equal(icompr!(x for let Some(x) in vec![Some(1), None, None, Some(2)]), vec![1, 2]);
    assert_equal(icompr!(() for let None in vec![Some(1), None, None, Some(2)]), vec![(), ()]);
    assert_equal(icompr!(x for let Some(x) in vec![Some(1), None, Some(7), Some(2)] if x > 2),
                 vec![7]);
}
