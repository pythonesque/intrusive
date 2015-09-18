//use std::fmt;
use std::intrinsics;
use std::mem;

use {Link, Base, Root};

pub enum DList<'id, T, S> where S: 'id, T: 'id {
    Empty,
    Base {
        next: Link<'id, DListNode<'id, T, S>, S>,
        prev: Link<'id, DListNode<'id, T, S>, S>,
    }
}

pub type DListNode<'id, T, S> where S: 'id, T: 'id = (T, DList<'id, T, S>);

/*impl<'id, T, S> fmt::Debug for DList<'id, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*try!(write!(f, "{:?}", self.0))*/
        Ok(())
    }
}*/

impl<'id, T, S> DList<'id, T, S> {
    pub fn empty() -> Self {
        DList::Empty
    }

    pub fn set_next<'a>(root: &mut Root<S>, node: Link<'id, DListNode<'id, T, S>, S>, next: Link<'id, DListNode<'id, T, S>, S>) {
        let old_next = match root[node].1 {
            DList::Base { next: ref mut old_next, .. } => mem::replace(old_next, next),
            ref mut old_node @ DList::Empty => {
                mem::replace(old_node, DList::Base { next: next, prev: node });
                node
            }
        };
        let old_prev = match root[next].1 {
            DList::Base { prev: ref mut old_prev, .. } => mem::replace(old_prev, node),
            ref mut old_node @ DList::Empty => {
                mem::replace(old_node, DList::Base { next: next, prev: node });
                next
            }
        };
        if let DList::Base { ref mut prev, .. } = root[old_next].1 {
            *prev = old_prev;
        } else {
            unsafe {
                // This should never actually happen as long as the list is consistent.
                intrinsics::abort();
            }
        };
        if let DList::Base { ref mut next, .. } = root[old_prev].1 {
            *next = old_next;
        } else {
            unsafe {
                // This should never actually happen as long as the list is consistent.
                intrinsics::abort();
            }
        };
    }

    pub fn next<'a>(root: &Root<S>, node: Link<'id, DListNode<'id, T, S>, S>) -> Link<'id, DListNode<'id, T, S>, S> {
        match root[node].1 {
            DList::Empty => node,
            DList::Base { next, .. } => next,
        }
    }

    pub fn prev<'a>(root: &Root<S>, node: Link<'id, DListNode<'id, T, S>, S>) -> Link<'id, DListNode<'id, T, S>, S> {
        match root[node].1 {
            DList::Empty => node,
            DList::Base { prev, .. } => prev,
        }
    }
}

#[cfg(test)]
mod tests {
    use {Root, Base};
    use super::{DList};

    #[test]
    fn safe_dlist_test() {
        Root::with( |mut root| {
            let (l_, r_);
            l_ = Base::new((
                (),
                DList::empty(),
            ));
            r_ = Base::new((
                (),
                DList::empty(),
            ));
            println!("{:?}", ::std::mem::size_of_val(&l_));
            let l = &l_;
            let r = &r_;

            DList::set_next(&mut root, l, r);

            println!("{:?}", &l as *const _);
            println!("{:?}", ::std::mem::size_of_val(&root[l]));
            println!("{:?}", &r as *const _);
        });
    }    

    #[test]
    fn safe_two_dlist_test() {
        Root::with( |mut root| {
            let (a_, b_, c_);
            a_ = Base::new((Base::new((vec!["1"], DList::empty())), DList::empty()));
            b_ = Base::new((Base::new((vec!["2"], DList::empty())), DList::empty()));
            c_ = Base::new((vec!["3"], DList::empty()));
            println!("{:?}", ::std::mem::size_of_val(&a_));
            let a = &a_;
            let b = &b_;

            DList::set_next(&mut root, a, b);

            let a2 = &**a;
            let b2 = &**b;
            let c = &c_;

            DList::set_next(&mut root, a2, b2);
            DList::set_next(&mut root, b2, c);

            {
                let x_ = Base::new((vec!["3"], DList::empty()));
                DList::set_next(&mut root, &x_, &x_);
            }

            //DList::set_next(&mut root, a, b);

            println!("a: {:?}, next: {:?}, prev: {:?}", root[&**a].0, root[&**DList::next(&root, a)].0, root[&**DList::prev(&root, a)].0);
            root[&**a].0.push("a");
            root[b2].0.push("b2");
            println!("a: {:?}", ::std::mem::size_of_val(&root[a]));
            println!("b: {:?}, next: {:?}, prev: {:?}", root[&**b].0, root[&**DList::next(&root, b)].0, root[&**DList::prev(&root, b)].0);
            println!("a2: {:?}, next: {:?}, prev: {:?}", root[a2].0, root[DList::next(&root, a2)].0, root[DList::prev(&root, a2)].0);
            println!("b2: {:?}", ::std::mem::size_of_val(&root[b2]));
            println!("b2: {:?}, next: {:?}, prev: {:?}", root[b2].0, root[DList::next(&root, b2)].0, root[DList::prev(&root, b2)].0);
            println!("c: {:?}, next: {:?}, prev: {:?}", root[c].0, root[DList::next(&root, c)].0, root[DList::prev(&root, c)].0);
        });
    }

    /*#[test]
    fn option_dlist_test() {
        Root::with( |mut root| {
            let a_ = Base::new(Some(Base::new((Base::new((vec!["1"], DList::empty())), DList::empty()))));
            if let Some(ref a) = root[&a_].take() {
                DList::set_next(&mut root, a, a);
                let b = &**a;
                DList::set_next(&mut root, b, b);

                println!("{:?}", root[b].0);
                /*DList::set_next(&mut root, a, a);
                DList::set_next(&mut root, b, b);*/
            } else {
                panic!();
            }
            //let 
        });
    }*/
}

