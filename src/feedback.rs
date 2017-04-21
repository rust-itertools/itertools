use std::cell::Cell;
use std::rc::Rc;

mod inner {
    use std::cell::Cell;
    use std::rc::Rc;

    pub struct Inner<T>(pub Rc<Cell<Option<T>>>);

    impl<T: Copy> Iterator for Inner<T> {
        type Item = T;

        fn next(&mut self) -> Option<T> {
            self.0.get()
        }
    }
}

/// See [`feedback`](../fn.feedback.html) for more information.
pub struct Feedback<I, T> {
    iter: I,
    inner: Rc<Cell<Option<T>>>,
}

/**
 * Feed the output of an iterator pipeline back to the input. Feedback
 * is delayed by one timestep to preserve causality, so the first input
 * is provided by `initial`, and the output of that pass is used as the
 * input for the second pass, and so on.
 *
 * Every time the input is requested, it yields the last result returned
 * by the pipeline. The pipeline can request the feedback value any
 * number of times per cycle, including ignoring it entirely. If the
 * pipeline doesn't request a particular input, that input is discarded,
 * not saved for the next cycle. If the pipeline requests an input
 * multiple times in the process of producing an output, the same input
 * will be returned each time.
 *
 * ```rust
 * use itertools::feedback;
 * let input = [1, -2, 3, -4, 5];
 * let result: Vec<i32> = feedback(0, |feedback|
 *         feedback.zip(&input)
 *                 .map(|(a, b)| a + b)
 *     ).collect();
 * assert_eq!(result, &[0, 1, -1, 2, -2, 3]);
 * ```
 */
pub fn feedback<F, I, T>(initial: T, inner: F) -> Feedback<I, T>
    where F: FnOnce(inner::Inner<T>) -> I,
          T: Copy
{
    let initial = Rc::new(Cell::new(Some(initial)));
    Feedback {
        iter: inner(inner::Inner(initial.clone())),
        inner: initial,
    }
}

impl<I, T> Iterator for Feedback<I, T>
    where I: Iterator<Item = T>,
          T: Copy
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if let Some(v) = self.inner.get() {
            self.inner.set(self.iter.next());
            Some(v)
        } else {
            None
        }
    }
}
