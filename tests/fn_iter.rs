extern crate itertools;

#[test]
fn test_fn_iter() {
    let nums = itertools::func({
        let mut counter = 0;
        move || { counter += 1; Some(counter) }
    });

    itertools::assert_equal(nums.take(10), 1..11);
}
