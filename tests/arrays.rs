use itertools::Itertools;

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
}
