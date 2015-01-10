use std::cell::RefCell;
use std::collections::RingBuf;
use std::rc::Rc;
use std::num::Int;

/// Common buffer object for the two tee halves
struct TeeBuffer<A, I>
{
    backlog: RingBuf<A>,
    iter: I,
    /// The owner field indicates which id should read from the backlog
    owner: bool,
}

/// One half of an iterator pair where both return the same elements.
pub struct Tee<A, I>
{
    rcbuffer: Rc<RefCell<TeeBuffer<A, I>>>,
    id: bool,
}

pub fn new<A, I>(iter: I) -> (Tee<A, I>, Tee<A, I>)
{
    let buffer = TeeBuffer{backlog: RingBuf::new(), iter: iter, owner: false};
    let t1 = Tee{rcbuffer: Rc::new(RefCell::new(buffer)), id: true};
    let t2 = Tee{rcbuffer: t1.rcbuffer.clone(), id: false};
    (t1, t2)
}

impl<A: Clone, I: Iterator<Item=A>> Iterator for Tee<A, I>
{
    type Item = A;
    fn next(&mut self) -> Option<A>
    {
        // .borrow_mut may fail here -- but only if the user has tied some kind of weird
        // knot where the iterator refers back to itself.
        //
        // Avoid panic!() in release code here for better code gen.
        let mut buffer = match self.rcbuffer.try_borrow_mut() {
            None => {
                debug_assert!(false, "Tee::next: Cycle in tee iterator.");
                return None;
            }
            Some(bufref) => bufref,
        };
        if buffer.owner == self.id {
            match buffer.backlog.pop_front() {
                None => {}
                some_elt => return some_elt,
            }
        }
        match buffer.iter.next() {
            None => None,
            Some(elt) => {
                buffer.backlog.push_back(elt.clone());
                buffer.owner = !self.id;
                Some(elt)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let buffer = self.rcbuffer.borrow();
        let (mut lower, mut upper) = buffer.iter.size_hint();

        if buffer.owner == self.id {
            let log_len = buffer.backlog.len();
            lower = lower.saturating_add(log_len);
            upper = upper.and_then(|x| x.checked_add(log_len));
        }
        (lower, upper)
    }
}
