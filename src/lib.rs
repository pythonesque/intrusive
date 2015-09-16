#![cfg_attr(test, feature(rustc_private))]
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops;

struct InvariantLifetime<'id>(
    PhantomData<*mut &'id ()>);

impl<'id> InvariantLifetime<'id> {
    #[inline]
    fn new() -> InvariantLifetime<'id> {
        InvariantLifetime(PhantomData)
    }
}

pub struct Node<'id, T> {
    inner_: UnsafeCell<T>,
    _marker: InvariantLifetime<'id>,
}

impl<'id, T> Node<'id, T> {
    #[inline]
    pub fn new(inner: T) -> Self {
        Node {
            inner_: UnsafeCell::new(inner),
            _marker: InvariantLifetime::new(),
        }
    }
}

pub type Link<'id, 'm, T> = &'m Node<'id, T>;

pub struct Root<'id>(InvariantLifetime<'id>);

impl<'id> Root<'id> {
    #[inline]
    pub fn with<F, T>(closure: F) -> T where
            F: for<'a> FnOnce(Root<'a>) -> T
    {
        closure(Root(InvariantLifetime::new()))
    }
}

impl<'id, 'm, T> ops::Index<Link<'id, 'm, T>> for Root<'id> {
    type Output = T;

    #[inline]
    fn index<'a>(&'a self, index: Link<'id, 'm, T>) -> &'a T {
        unsafe {
            &*index.inner_.get()
        }
    }
}

impl<'id, 'm, T> ops::IndexMut<Link<'id, 'm, T>> for Root<'id> {
    #[inline]
    fn index_mut<'a>(&'a mut self, index: Link<'id, 'm, T>) -> &'a mut T {
        unsafe {
            &mut *index.inner_.get()
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate arena;
    use self::arena::TypedArena;
    use {Link, Node, Root};

    #[test]
    fn it_works() {
        struct Foo<'id, 'm, T> where T: 'm, 'id: 'm {
            data: T,
            link_a: Option<Link<'id, 'm, Foo<'id, 'm, T>>>,
            link_b: Option<Link<'id, 'm, Foo<'id, 'm, T>>>,
        }

        Root::with(|mut root| /*Root::with(|mut root_a| Root::with(|mut root_b|*/ {
            let mut data1 = 1u8;
            let mut data2 = 2u8;
            let mut data3 = 3u8;

            let (node1, node2, node3);

            node2 = Node::new(Foo { data: (2u8, Some(&mut data2)), link_a: None, link_b: None });
            node3 = Node::new(Foo { data: (3u8, Some(&mut data3)), link_a: None, link_b: Some(&node2) });
            node1 = Node::new(Foo { data: (1u8, Some(&mut data1)), link_a: Some(&node2), link_b: Some(&node3) });

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

        }/*))*/);

        Root::with(|mut root| {
            let mut data = 0u8;
            let mut data_ = 1u8;
            let mut data__ = 2u8;
            let (node, node_);

            struct Bar<'id, 'm, T> where T: 'm, 'id: 'm {
                data: T,
                next: Option<Link<'id, 'm, Bar<'id, 'm, T>>>,
            };

            node_ = Node::new(Bar { data: (1u8, Some(&mut data_)), next: None });
            node = Node::new(Bar { data: (0u8, Some(&mut data)), next: Some(&node_)});

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
                let node_ = Node::new(Bar {
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

        struct Set<'id, 'm, T> where T: 'm, 'id: 'm {
            data: T,
            parent: Option<Link<'id, 'm, Set<'id, 'm, T>>>,
            rank: u8,
        }

        impl<'id, 'm, T> Set<'id, 'm, T> {
            fn make(data: T) -> Self {
                Set {
                    data: data,
                    parent: None,
                    rank: 0,
                }
            }

            fn find(root: &mut Root<'id>, x: Link<'id, 'm, Self>) -> Link<'id, 'm, Self> {
                match root[x].parent {
                    Some(parent) => {
                        let parent = Set::find(root, parent);
                        root[x].parent = Some(parent);
                        parent
                    },
                    None => x,
                }
            }

            fn union(root: &mut Root<'id>, x: Link<'id, 'm, Self>, y: Link<'id, 'm, Self>) {
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

            let x = arena.alloc(Node::new(Set::make("x".to_string())));
            let y = arena.alloc(Node::new(Set::make("y".into())));
            let z = arena.alloc(Node::new(Set::make("z".into())));

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
        });
    }
}
