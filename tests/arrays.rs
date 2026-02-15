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

    // Check that array_windows agrees with tuple_windows
    assert_eq!(
        vec4.iter().copied().array_windows().collect::<Vec<_>>(),
        vec4.iter()
            .copied()
            .tuple_windows()
            .map(|(a,)| [a])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec4.iter().copied().array_windows().collect::<Vec<_>>(),
        vec4.iter()
            .copied()
            .tuple_windows()
            .map(|(a, b)| [a, b])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec4.iter().copied().array_windows().collect::<Vec<_>>(),
        vec4.iter()
            .copied()
            .tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec2.iter().copied().array_windows().collect::<Vec<_>>(),
        vec2.iter()
            .copied()
            .tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec1.iter().copied().array_windows().collect::<Vec<_>>(),
        vec1.iter()
            .copied()
            .tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec0.iter().copied().array_windows().collect::<Vec<_>>(),
        vec0.iter()
            .copied()
            .tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );
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

    // Check that circular_array_windows agrees with circular_tuple_windows
    assert_eq!(
        vec4.iter()
            .copied()
            .circular_array_windows()
            .collect::<Vec<_>>(),
        vec4.iter()
            .copied()
            .circular_tuple_windows()
            .map(|(a,)| [a])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec4.iter()
            .copied()
            .circular_array_windows()
            .collect::<Vec<_>>(),
        vec4.iter()
            .copied()
            .circular_tuple_windows()
            .map(|(a, b)| [a, b])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec4.iter()
            .copied()
            .circular_array_windows()
            .collect::<Vec<_>>(),
        vec4.iter()
            .copied()
            .circular_tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec2.iter()
            .copied()
            .circular_array_windows()
            .collect::<Vec<_>>(),
        vec2.iter()
            .copied()
            .circular_tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec1.iter()
            .copied()
            .circular_array_windows()
            .collect::<Vec<_>>(),
        vec1.iter()
            .copied()
            .circular_tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        vec0.iter()
            .copied()
            .circular_array_windows()
            .collect::<Vec<_>>(),
        vec0.iter()
            .copied()
            .circular_tuple_windows()
            .map(|(a, b, c, d, e, f, g)| [a, b, c, d, e, f, g])
            .collect::<Vec<_>>(),
    );
}
