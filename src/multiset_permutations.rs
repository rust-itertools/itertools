use std::iter::FromIterator;

/// TODO!
#[derive(Debug, Clone)]
pub struct MultisetPermutations<I> {
    buffer: Vec<Node<I>>,
    start: bool,
    head: usize,
    index: usize,
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
        index: length.saturating_sub(2),
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
            let mut permutation = Vec::with_capacity(self.buffer.len());
            let mut curr = self.head;
            for _ in 0..self.buffer.len() {
                permutation.push(self.buffer[curr].value);
                match self.buffer[curr].next {
                    Some(next) => curr = next,
                    None => break,
                }
            }

            return Some(permutation);
        }

        // In case of empty buffer
        if self.buffer.len() <= self.index {
            return None;
        }

        let next = match self.buffer[self.index].next {
            Some(next) => next,
            None => return None,
        };

        if self.buffer[next].next.is_none() {
            if self.buffer[self.head].value <= self.buffer[next].value {
                return None;
            }
        } else {
            let next_next = self.buffer[next].next.unwrap();
            let shift_index = if self.buffer[next_next].value <= self.buffer[self.index].value {
                next_next
            } else {
                next
            };

            

        }

        return None;
        // // [0,1,2,3,4,5,6,7,8,9,10,0] 4.15s 239500800 base
        // // [0,1,2,3,4,5,6,7,8,9,10,0] 4.44s 239500800 opt1
        // // [0,1,2,3,4,5,6,7,8,9,10,0] 3.4 239500800 opt1
        // let elem = self.buffer[shift_index];
        // let mut i = shift_index;
        // while i > 0 {
        //     self.buffer[i] = self.buffer[i - 1];
        //     i -= 1;
        // }
        // // for i in (0..shift_index).rev() {
        // //     self.buffer[i + 1] = self.buffer[i]
        // // }
        // self.buffer[0] = elem;

        // // let shift_element = self.buffer.remove(shift_index);
        // // self.buffer.insert(0, shift_element);

        // if self.buffer[0] < self.buffer[1] {
        //     self.index = 0;
        // } else {
        //     self.index += 1;
        // }

        // Some(self.buffer.clone())
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
