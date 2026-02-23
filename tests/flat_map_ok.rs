use itertools::{assert_equal, Itertools};
use std::vec::IntoIter;

fn mix_data() -> IntoIter<Result<i32, bool>> {
    vec![Ok(2), Err(false), Ok(3), Err(true), Ok(0)].into_iter()
}

fn ok_data() -> IntoIter<Result<i32, bool>> {
    vec![Ok(2), Ok(3), Ok(0)].into_iter()
}

#[test]
fn flat_map_ok_mixed_forward() {
    assert_equal(
        mix_data().flat_map_ok(|i| 0..i),
        vec![
            Ok(0),
            Ok(1),
            Err(false),
            Ok(0),
            Ok(1),
            Ok(2),
            Err(true),
        ],
    );
}

#[test]
fn flat_map_ok_mixed_reverse() {
    assert_equal(
        mix_data().flat_map_ok(|i| 0..i).rev(),
        vec![
            Err(true),
            Ok(2),
            Ok(1),
            Ok(0),
            Err(false),
            Ok(1),
            Ok(0),
        ],
    );
}

#[test]
fn flat_map_ok_collect_mixed() {
    assert_eq!(
        mix_data()
            .flat_map_ok(|i| 0..i)
            .collect::<Result<Vec<_>, _>>(),
        Err(false)
    );
}

#[test]
fn flat_map_ok_collect_ok_forward() {
    assert_eq!(
        ok_data()
            .flat_map_ok(|i| 0..i)
            .collect::<Result<Vec<_>, _>>(),
        Ok(vec![0, 1, 0, 1, 2])
    );
}

#[test]
fn flat_map_ok_collect_ok_reverse() {
    assert_eq!(
        ok_data()
            .flat_map_ok(|i| 0..i)
            .rev()
            .collect::<Result<Vec<_>, _>>(),
        Ok(vec![2, 1, 0, 1, 0])
    );
}

#[test]
fn flat_map_ok_empty_results() {
    // When the mapping function returns an empty iterator for some Ok values
    let data: Vec<Result<i32, bool>> = vec![Ok(0), Ok(2), Ok(0)];
    assert_equal(
        data.into_iter().flat_map_ok(|i| 0..i),
        vec![Ok(0), Ok(1)],
    );
}

#[test]
fn flat_map_ok_all_errors() {
    let data: Vec<Result<i32, bool>> = vec![Err(false), Err(true)];
    assert_equal(
        data.into_iter().flat_map_ok(|i: i32| 0..i),
        vec![Err(false), Err(true)],
    );
}

#[test]
fn flat_map_ok_equivalence_with_map_ok_flatten_ok() {
    // flat_map_ok(f) should be equivalent to map_ok(f).flatten_ok()
    let data1: Vec<Result<i32, bool>> = vec![Ok(2), Err(false), Ok(3)];
    let data2 = data1.clone();

    let result1: Vec<_> = data1.into_iter().flat_map_ok(|i| 0..i).collect();
    let result2: Vec<_> = data2.into_iter().map_ok(|i| 0..i).flatten_ok().collect();

    assert_eq!(result1, result2);
}
