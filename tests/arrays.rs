use itertools::Itertools;

#[test]
fn array_windows() {
    let [vec0, vec1, vec2, vec4, vec10] = [
        vec![],
        vec![1],
        vec![1, 2],
        vec![1, 2, 3, 4],
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    ];

    assert_eq!(
        vec10
            .iter()
            .copied()
            .array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 6],
            [6, 7],
            [7, 8],
            [8, 9],
            [9, 10],
        ]
    );

    assert_eq!(
        vec4.iter()
            .copied()
            .array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![[1, 2], [2, 3], [3, 4]]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![[1, 2]]
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .array_windows::<2>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 2]>::new()
    );

    assert_eq!(
        vec0.iter()
            .copied()
            .array_windows::<2>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 2]>::new()
    );

    assert_eq!(
        vec10
            .iter()
            .copied()
            .array_windows::<4>()
            .collect::<Vec<_>>(),
        vec![
            [1, 2, 3, 4],
            [2, 3, 4, 5],
            [3, 4, 5, 6],
            [4, 5, 6, 7],
            [5, 6, 7, 8],
            [6, 7, 8, 9],
            [7, 8, 9, 10],
        ]
    );

    // For zero-length output windows, the equation
    //
    //   output length = input length + N - 1
    //
    // implies that we return one _more_ zero-length window than there
    // are input items, as if we were returning a zero-length window
    // for each position between elements of the input list, including
    // the positions at the start and end.
    assert_eq!(
        vec0.iter()
            .copied()
            .array_windows::<0>()
            .collect::<Vec<_>>(),
        vec![[]]
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .array_windows::<0>()
            .collect::<Vec<_>>(),
        vec![[], []]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .array_windows::<0>()
            .collect::<Vec<_>>(),
        vec![[], [], []]
    );

    assert_eq!(
        vec0.iter()
            .copied()
            .array_windows::<1>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 1]>::new()
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .array_windows::<1>()
            .collect::<Vec<_>>(),
        vec![[1]]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .array_windows::<1>()
            .collect::<Vec<_>>(),
        vec![[1], [2]]
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .array_windows::<7>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 7]>::new()
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .array_windows::<7>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 7]>::new()
    );

    assert_eq!(
        vec4.iter()
            .copied()
            .array_windows::<7>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 7]>::new()
    );
}

#[test]
fn array_windows_equal_tuple_windows() {
    fn internal(slice: &[usize]) {
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_,)>()
                .map(Into::<[_; 1]>::into),
            slice.iter().array_windows::<1>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _,)>()
                .map(Into::<[_; 2]>::into),
            slice.iter().array_windows::<2>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _,)>()
                .map(Into::<[_; 3]>::into),
            slice.iter().array_windows::<3>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _,)>()
                .map(Into::<[_; 4]>::into),
            slice.iter().array_windows::<4>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _, _,)>()
                .map(Into::<[_; 5]>::into),
            slice.iter().array_windows::<5>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _, _, _,)>()
                .map(Into::<[_; 6]>::into),
            slice.iter().array_windows::<6>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _, _, _, _,)>()
                .map(Into::<[_; 7]>::into),
            slice.iter().array_windows::<7>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _, _, _, _, _,)>()
                .map(Into::<[_; 8]>::into),
            slice.iter().array_windows::<8>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _, _, _, _, _, _,)>()
                .map(Into::<[_; 9]>::into),
            slice.iter().array_windows::<9>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .tuple_windows::<(_, _, _, _, _, _, _, _, _, _,)>()
                .map(Into::<[_; 10]>::into),
            slice.iter().array_windows::<10>()
        );
    }
    for i in 0..100 {
        internal(&(0..i).collect::<Vec<_>>());
    }
}

#[test]
fn circular_array_windows() {
    let [vec0, vec1, vec2, vec4, vec10] = [
        vec![],
        vec![1],
        vec![1, 2],
        vec![1, 2, 3, 4],
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    ];

    assert_eq!(
        vec10
            .iter()
            .copied()
            .circular_array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 6],
            [6, 7],
            [7, 8],
            [8, 9],
            [9, 10],
            [10, 1]
        ]
    );

    assert_eq!(
        vec4.iter()
            .copied()
            .circular_array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![[1, 2], [2, 3], [3, 4], [4, 1]]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .circular_array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![[1, 2], [2, 1]]
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .circular_array_windows::<2>()
            .collect::<Vec<_>>(),
        vec![[1, 1]]
    );

    assert_eq!(
        vec0.iter()
            .copied()
            .circular_array_windows::<2>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 2]>::new()
    );

    assert_eq!(
        vec10
            .iter()
            .copied()
            .circular_array_windows::<4>()
            .collect::<Vec<_>>(),
        vec![
            [1, 2, 3, 4],
            [2, 3, 4, 5],
            [3, 4, 5, 6],
            [4, 5, 6, 7],
            [5, 6, 7, 8],
            [6, 7, 8, 9],
            [7, 8, 9, 10],
            [8, 9, 10, 1],
            [9, 10, 1, 2],
            [10, 1, 2, 3],
        ]
    );

    // For zero-length output windows, we preserve the invariant that
    // output length = input length, and return the same number of
    // zero-length windows as there were input items, as if we were
    // returning a zero-length window for every possible starting
    // position in the cyclic input list.
    assert_eq!(
        vec0.iter()
            .copied()
            .circular_array_windows::<0>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 0]>::new()
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .circular_array_windows::<0>()
            .collect::<Vec<_>>(),
        vec![[]]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .circular_array_windows::<0>()
            .collect::<Vec<_>>(),
        vec![[], []]
    );

    assert_eq!(
        vec0.iter()
            .copied()
            .circular_array_windows::<1>()
            .collect::<Vec<_>>(),
        Vec::<[i32; 1]>::new()
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .circular_array_windows::<1>()
            .collect::<Vec<_>>(),
        vec![[1]]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .circular_array_windows::<1>()
            .collect::<Vec<_>>(),
        vec![[1], [2]]
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .circular_array_windows::<7>()
            .collect::<Vec<_>>(),
        vec![[1, 1, 1, 1, 1, 1, 1]]
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .circular_array_windows::<7>()
            .collect::<Vec<_>>(),
        vec![[1, 2, 1, 2, 1, 2, 1], [2, 1, 2, 1, 2, 1, 2]]
    );

    assert_eq!(
        vec4.iter()
            .copied()
            .circular_array_windows::<7>()
            .collect::<Vec<_>>(),
        vec![
            [1, 2, 3, 4, 1, 2, 3],
            [2, 3, 4, 1, 2, 3, 4],
            [3, 4, 1, 2, 3, 4, 1],
            [4, 1, 2, 3, 4, 1, 2],
        ]
    );
}

#[test]
fn circular_array_windows_equal_circular_tuple_windows() {
    fn internal(slice: &[usize]) {
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_,)>()
                .map(Into::<[_; 1]>::into),
            slice.iter().circular_array_windows::<1>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _,)>()
                .map(Into::<[_; 2]>::into),
            slice.iter().circular_array_windows::<2>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _,)>()
                .map(Into::<[_; 3]>::into),
            slice.iter().circular_array_windows::<3>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _,)>()
                .map(Into::<[_; 4]>::into),
            slice.iter().circular_array_windows::<4>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _, _,)>()
                .map(Into::<[_; 5]>::into),
            slice.iter().circular_array_windows::<5>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _, _, _,)>()
                .map(Into::<[_; 6]>::into),
            slice.iter().circular_array_windows::<6>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _, _, _, _,)>()
                .map(Into::<[_; 7]>::into),
            slice.iter().circular_array_windows::<7>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _, _, _, _, _,)>()
                .map(Into::<[_; 8]>::into),
            slice.iter().circular_array_windows::<8>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _, _, _, _, _, _,)>()
                .map(Into::<[_; 9]>::into),
            slice.iter().circular_array_windows::<9>()
        );
        itertools::assert_equal(
            slice
                .iter()
                .circular_tuple_windows::<(_, _, _, _, _, _, _, _, _, _,)>()
                .map(Into::<[_; 10]>::into),
            slice.iter().circular_array_windows::<10>()
        );
    }
    for i in 0..100 {
        internal(&(0..i).collect::<Vec<_>>());
    }
}
