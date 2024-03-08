use std::iter::FromIterator;

/// TODO!
#[derive(Debug, Clone)]
pub struct MultisetPermutations<I> {
    buffer: Vec<Node<I>>,
    start: bool,
    head: usize,
    next: usize,
    next_next: usize,
}

#[derive(Debug, Clone)]
struct Node<I> {
    value: I,
    next: Option<usize>,
}

pub fn multiset_permutations<I: Iterator>(iter: I) -> MultisetPermutations<I::Item>
where
    I: Iterator,
    I::Item: Ord,
{
    let mut buffer = Vec::from_iter(iter);
    buffer.sort_unstable_by(|a, b| b.cmp(a));
    let length = buffer.len();
    MultisetPermutations {
        buffer: buffer
            .into_iter()
            .enumerate()
            .map(|(curr, v)| Node {
                value: v,
                next: if curr + 1 < length {
                    Some(curr + 1)
                } else {
                    None
                },
            })
            .collect(),
        start: true,
        head: 0,
        next: length.saturating_sub(2),
        next_next: length.saturating_sub(1),
    }
}

impl<I: Copy> Iterator for MultisetPermutations<I>
where
    I: Ord,
{
    type Item = Vec<I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start {
            self.start = false;
            return Some(self.get_permutation())
        }

        // Special cases
        if self.buffer.len() <= 1 {
            return None;
        }

        // Finish condition
        let is_last = self.buffer[self.next_next].next.is_none();
        if is_last && self.buffer[self.head].value <= self.buffer[self.next_next].value {
            return None;
        }

        // Prefix shift
        let shift = if !is_last
            && self.buffer[self.buffer[self.next_next].next.unwrap()].value
                <= self.buffer[self.next].value
        {
            self.next_next
        } else {
            self.next
        };
        let shift_next = self.buffer[shift].next.unwrap();
        self.buffer[shift].next = self.buffer[shift_next].next;
        self.buffer[shift_next].next = Some(self.head);

        // Update pointers
        if self.buffer[shift_next].value < self.buffer[self.head].value {
            self.next = shift_next;
        }
        self.next_next = self.buffer[self.next].next.unwrap();
        self.head = shift_next;

        Some(self.get_permutation())
    }
}

impl<I: Copy> MultisetPermutations<I> {
    fn get_permutation(&self) -> Vec<I> {
        let mut permutation = Vec::with_capacity(self.buffer.len());
        let mut curr = Some(self.head);
        while curr.is_some() {
            let Node {value, next } = self.buffer[curr.unwrap()];
            permutation.push(value);
            curr = next;
        }
        permutation
    }
}

#[cfg(test)]
mod tests {
    use crate::Itertools;

    #[test]
    fn test1() {
        let mut iter = vec![1, 4, 2, 1].into_iter().multiset_permutations();
        assert_eq!(iter.next(), Some(vec![4, 2, 1, 1]));
        assert_eq!(iter.next(), Some(vec![1, 4, 2, 1]));
        assert_eq!(iter.next(), Some(vec![4, 1, 2, 1]));
        assert_eq!(iter.next(), Some(vec![1, 4, 1, 2]));
        assert_eq!(iter.next(), Some(vec![1, 1, 4, 2]));
        assert_eq!(iter.next(), Some(vec![4, 1, 1, 2]));
        assert_eq!(iter.next(), Some(vec![2, 4, 1, 1]));
        assert_eq!(iter.next(), Some(vec![1, 2, 4, 1]));
        assert_eq!(iter.next(), Some(vec![2, 1, 4, 1]));
        assert_eq!(iter.next(), Some(vec![1, 2, 1, 4]));
        assert_eq!(iter.next(), Some(vec![1, 1, 2, 4]));
        assert_eq!(iter.next(), Some(vec![2, 1, 1, 4]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test7() {
        let mut iter: crate::MultisetPermutations<i32> = vec![].into_iter().multiset_permutations();
        assert_eq!(iter.next(), Some(vec![]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn timing() {
        use std::time::Instant;
        let now = Instant::now();

        let iter = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0]
            .iter()
            .multiset_permutations();
        let count = iter.count();

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?} {count}", elapsed);
    }
}
