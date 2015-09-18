#![feature(associated_consts)]
#![feature(const_fn, core_intrinsics)]
#![feature(thread_local)]
#![cfg_attr(test, feature(rustc_private))]
#![feature(optin_builtin_traits)]

use std::cell::{Cell, UnsafeCell};
use std::intrinsics;
use std::marker::PhantomData;
use std::ops;

//pub mod dlist;
pub mod safe_dlist;
//mod hlist;
//mod graph;
//mod tree;

struct GlobalLock(UnsafeCell<u8>);

impl GlobalLock {
    const fn new() -> Self {
        GlobalLock(UnsafeCell::new(0))
    }
}

unsafe impl Sync for GlobalLock {}

static GLOBAL_LOCK: GlobalLock = GlobalLock::new();

struct ThreadLock(Cell<u8>);

impl ThreadLock {
    const fn new() -> Self {
        ThreadLock(Cell::new(0))
    }
}

unsafe impl Sync for ThreadLock {}

#[thread_local]
static THREAD_LOCK: ThreadLock = ThreadLock::new();

struct InvariantLifetime<'id>(
    PhantomData<*mut &'id ()>);

impl<'id> InvariantLifetime<'id> {
    #[inline]
    const fn new() -> InvariantLifetime<'id> {
        InvariantLifetime(PhantomData)
    }
}

pub struct Base<T, S> {
    inner: UnsafeCell<T>,
    _marker: PhantomData<S>,
}

pub struct GlobalGuard(());

pub struct ThreadGuard(());

impl !Send for ThreadGuard {}

unsafe impl<T, S> Send for Base<T, S> where T: Send, S: Send {}
unsafe impl<T, S> Sync for Base<T, S> where T: Sync, S: Sync {}

impl<T, S> Base<T, S> {
    #[inline]
    pub const fn new(inner: T) -> Self {
        Base {
            inner: UnsafeCell::new(inner),
            _marker: PhantomData,
        }
    }
}

impl<T, U, S> ops::Deref for Base<(Base<T, S>, U), S> {
    type Target = Base<T, S>;

    #[inline]
    fn deref<'id>(&'id self) -> &'id Base<T, S> {
        unsafe {
            //let (ref first, ref second) = *index.inner.get();
            //(first, second)
            //let (ref first, _) = *index.inner.get();
            &(&*self.inner.get()).0
        }
    }
}

pub type Link<'id, T, S> = &'id Base<T, S>;

/*impl<T, U, S> Base<(Base<T, S>, U), S> {
    #[inline]
    pub fn upcast<'id>(&'id self) -> Link<'id, T, S>
    {
        unsafe {
            //let (ref first, ref second) = *index.inner.get();
            //(first, second)
            //let (ref first, _) = *index.inner.get();
            &(&*self.inner.get()).0
        }
    }
}*/

pub struct Root<S>(S);

impl<'id> Root<InvariantLifetime<'id>> {
    #[inline]
    pub fn with<F, T>(closure: F) -> T where
            F: for<'a> FnOnce(Root<InvariantLifetime<'a>>) -> T
    {
        closure(Root(InvariantLifetime::new()))
    }
}

impl Root<GlobalGuard> {
    pub fn global() -> Self {
        unsafe {
            if intrinsics::atomic_xchg_acq(GLOBAL_LOCK.0.get(), !0) != 0 {
                intrinsics::abort();
            }
            Root(GlobalGuard(()))
        }
    }
}

impl Drop for GlobalGuard {
    fn drop(&mut self) {
        unsafe {
            intrinsics::atomic_store_rel(GLOBAL_LOCK.0.get(), 0);
        }
    }
}

impl Root<ThreadGuard> {
    pub fn thread() -> Self {
        if THREAD_LOCK.0.get() != 0 {
            unsafe {
                intrinsics::abort();
            }
        }
        THREAD_LOCK.0.set(!0);
        Root(ThreadGuard(()))
    }
}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        THREAD_LOCK.0.set(0);
    }
}

impl<'id, T, S> ops::Index<Link<'id, T, S>> for Root<S> {
    type Output = T;

    #[inline]
    fn index<'a>(&'a self, index: Link<'id, T, S>) -> &'a T {
        unsafe {
            &*index.inner.get()
        }
    }
}

impl<'id, T, S> ops::IndexMut<Link<'id, T, S>> for Root<S> {
    #[inline]
    fn index_mut<'a>(&'a mut self, index: Link<'id, T, S>) -> &'a mut T {
        unsafe {
            &mut *index.inner.get()
        }
    }
}

/*impl<S> Root<S> {
    /*#[inline]
    pub fn split_at<'a, 'id, T, U>(&'a self, index: Link<'id, (Base<T, S>, U), S>) -> (Link<'id, T, S>, &'a U) //where
            //T: 'id,
            //T: 'a,
            //U: 'id,
            //V: 'id,
            //'id: 'a,
            //'a: 'id,
            //U: 'a,
            //'a: 'id,
    {
        unsafe {
            let (ref first, ref second) = *index.inner.get();
            (first, second)
        }
    }*/

    #[inline]
    pub fn upcast<'a, 'id, T, U>(&'a self, index: Link<'id, (Base<T, S>, U), S>) -> Link<'id, T, S>
    {
        unsafe {
            //let (ref first, ref second) = *index.inner.get();
            //(first, second)
            //let (ref first, _) = *index.inner.get();
            &(&*index.inner.get()).0
        }
    }

    /*#[inline]
    pub fn split_at_mut<'a, 'id, T, U>(&'a self, index: Link<'id, (Base<T, S>, U), S>) -> (Link<'id, T, S>, &'a mut U) //where
            //T: 'id,
            //T: 'a,
            //U: 'id,
            //V: 'id,
            //'id: 'a,
            //'a: 'id,
            //U: 'a,
            //'a: 'id,
    {
        unsafe {
            let (ref mut first, ref mut second) = *index.inner.get();
            (first, second)
        }
    }*/
}*/

#[cfg(test)]
mod tests {
    extern crate arena;
    use self::arena::TypedArena;
    use {GlobalGuard, Link, Base, Root};

    #[test]
    fn it_works() {
        struct Foo<'id, T, S> where T: 'id, S: 'id {
            data: T,
            link_a: Option<Link<'id, Foo<'id, T, S>, S>>,
            link_b: Option<Link<'id, Foo<'id, T, S>, S>>,
        }

        Root::with(|mut root| {
            let mut data1 = 1u8;
            let mut data2 = 2u8;
            let mut data3 = 3u8;

            let (node1, node2, node3);

            node2 = Base::new(Foo { data: (2u8, Some(&mut data2)), link_a: None, link_b: None });
            node3 = Base::new(Foo { data: (3u8, Some(&mut data3)), link_a: None, link_b: Some(&node2) });
            node1 = Base::new(Foo { data: (1u8, Some(&mut data1)), link_a: Some(&node2), link_b: Some(&node3) });

            root[&node2].link_a = Some(&node3);
            root[&node2].link_b = Some(&node1);
            root[&node3].link_a = Some(&node1);

            println!("node1: {:?}", root[&node1].data);
            let next_a = root[&node1].link_a.unwrap();
            println!("node1 next_a: {:?}", root[next_a].data);
            let next_b = root[&node1].link_b.unwrap();
            println!("node1 next_b: {:?}", root[next_b].data);
            println!("node2: {:?}", root[&node2].data);
            let next_a = root[&node2].link_a.unwrap();
            println!("node2 next_a: {:?}", root[next_a].data);
            let next_b = root[&node2].link_b.unwrap();
            println!("node2 next_b: {:?}", root[next_b].data);
            println!("node3: {:?}", root[&node3].data);
            let next_a = root[&node3].link_a.unwrap();
            println!("node3 next_a: {:?}", root[next_a].data);
            let next_b = root[&node3].link_b.unwrap();
            println!("node3 next_b: {:?}", root[next_b].data);

        });

        Root::with(|mut root| {
            let mut data = 0u8;
            let mut data_ = 1u8;
            let mut data__ = 2u8;
            let (node, node_);

            struct Bar<'id, T, S> where T: 'id, S: 'id {
                data: T,
                next: Option<Link<'id, Bar<'id, T, S>, S>>,
            };

            node_ = Base::new(Bar { data: (1u8, Some(&mut data_)), next: None });
            node = Base::new(Bar { data: (0u8, Some(&mut data)), next: Some(&node_)});

            root[&node_].next = Some(&node);

            Root::with(|mut _root| {
                {
                    let node1 = &mut root[&node];
                    //let node2 = _root[&node_];
                    if let Some(ref mut x) = node1.data.1 {
                        //node2.1 = None;
                        println!("{:?}", x);
                    }
                }
            });

            {
                //let data__ = root[&node].data.0;
                let node_ = Base::new(Bar {
                    data: (1u8, Some(&mut data__)),
                    next: None
                });

                root[&node_].next = Some(&node);
                //root[&node].next = Some(&node_);
                let next_ = root[&node_].next.unwrap();
                println!("inner node_: {:?}", root[&node_].data);
                println!("inner node_.next: {:?}", root[next_].data);
            }

            root[&node].next = Some(&node_);

            if let Some(ref mut x) = root[&node].data.1 {
                **x = 7;
            }

            let next = root[&node].next.unwrap();
            println!("node: {:?}", root[&node].data);
            println!("node.next: {:?}", root[next].data);
            let next_ = root[&node_].next.unwrap();
            println!("node_: {:?}", root[&node_].data);
            println!("node_.next: {:?}", root[next_].data);
        });

        let mut bar = 1;
        let mut foo = (&mut bar, 2);
        foo.0 = &mut foo.1;
        *foo.0 = 3;
    }

    #[test]
    fn union_find() {
        use std::cmp::Ordering;

        struct Set<'id, T, S> where T: 'id, S: 'id {
            data: T,
            parent: Option<Link<'id, Set<'id, T, S>, S>>,
            rank: u8,
        }

        impl<'id, T, S> Set<'id, T, S> {
            const fn make(data: T) -> Self {
                Set {
                    data: data,
                    parent: None,
                    rank: 0,
                }
            }

            fn find(root: &mut Root<S>, x: Link<'id, Self, S>) -> Link<'id, Self, S> {
                match root[x].parent {
                    Some(parent) => {
                        let parent = Set::find(root, parent);
                        root[x].parent = Some(parent);
                        parent
                    },
                    None => x,
                }
            }

            fn union(root: &mut Root<S>, x: Link<'id, Self, S>, y: Link<'id, Self, S>) {
                let x_root = Set::find(root, x);
                let y_root = Set::find(root, y);
                
                if x_root as *const _ == y_root as *const _ { return }

                match root[x_root].rank.cmp(&root[y_root].rank) {
                    Ordering::Less => {
                        root[x_root].parent = Some(y_root);
                    },
                    Ordering::Greater => {
                        root[y_root].parent = Some(x_root);
                    },
                    Ordering::Equal => {
                        root[y_root].parent = Some(x_root);
                        root[x_root].rank += 1;
                    }
                }
            }
        }
        Root::with(|mut root| {
            let arena = TypedArena::new();

            let x = arena.alloc(Base::new(Set::make("x".to_string())));
            let y = arena.alloc(Base::new(Set::make("y".into())));
            let z = arena.alloc(Base::new(Set::make("z".into())));

            let x_ = Set::find(&mut root, x);
            let y_ = Set::find(&mut root, y);
            let z_ = Set::find(&mut root, z);
            root[x_].data.push_str("_append1");
            println!("x: {:?} y: {:?}, z: {:?}", root[x_].data, root[y_].data, root[z_].data);

            Set::union(&mut root, x, y);
            let x_ = Set::find(&mut root, x);
            let y_ = Set::find(&mut root, y);
            let z_ = Set::find(&mut root, z);
            root[y_].data.push_str("_append2");
            println!("x: {:?} y: {:?}, z: {:?}", root[x_].data, root[y_].data, root[z_].data);

            Set::union(&mut root, x, z);
            let x_ = Set::find(&mut root, x);
            let y_ = Set::find(&mut root, y);
            let z_ = Set::find(&mut root, z);
            root[z_].data.push_str("_append3");
            println!("x: {:?} y: {:?}, z: {:?}", root[x_].data, root[y_].data, root[z_].data);

            type SSet = Base<Set<'static, &'static str, GlobalGuard>, GlobalGuard>;
            static NODES: [SSet; 3] = [Base::new(Set::make("")), Base::new(Set::make("")), Base::new(Set::make(""))];

            let w = {
                let mut root = Root::global();

                let x = &NODES[0];
                let y = &NODES[1];
                let z = &NODES[2];

                root[x].data = "x";
                root[y].data = "y";
                root[z].data = "z";

                let x_ = Set::find(&mut root, x);
                let y_ = Set::find(&mut root, y);
                let z_ = Set::find(&mut root, z);
                println!("x: {:?} y: {:?}, z: {:?}", root[x_].data, root[y_].data, root[z_].data);

                Set::union(&mut root, x, y);
                let x_ = Set::find(&mut root, x);
                let y_ = Set::find(&mut root, y);
                let z_ = Set::find(&mut root, z);
                println!("x: {:?} y: {:?}, z: {:?}", root[x_].data, root[y_].data, root[z_].data);

                Set::union(&mut root, x, z);
                let x_ = Set::find(&mut root, x);
                let y_ = Set::find(&mut root, y);
                let z_ = Set::find(&mut root, z);
                println!("x: {:?} y: {:?}, z: {:?}", root[x_].data, root[y_].data, root[z_].data);

                //let (w, v);
                let w = Base::new(Set::make(""));
                //v = Base::new(Set::make(""));
                root[&w].parent = Some(y);
                //root[&v].parent = Some(&w);
                println!("{:?}", root[&w].data);
                w
            };
            let mut root = Root::global();

            root[&w].data = "3";

            println!("{:?}", root[&w].data);
        });
    }
}
