// DONE: implement append.
//      Goof: I was attempting to iterate the list using `Box` without a reference.
//      `Box` uniquely owns the pointer.
//      let b = Box::new(123);
//      let b2 = b; // b is moved and cannot be used.
// DONE: implement len.
// DONE: implement Display trait.
// DONE: implement foreach.
// DONE: implement iterator.

use std::fmt;
use std::iter::Iterator;

struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

struct LinkedList<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> std::fmt::Display for LinkedList<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tail_ref = &self.root;
        while tail_ref.is_some() {
            write!(f, "{} => ", tail_ref.as_ref().unwrap().data)?;
            tail_ref = &tail_ref.as_ref().unwrap().next;
        }
        write!(f, "(None)")
    }
}

impl<T> LinkedList<T> {
    fn new() -> LinkedList<T> {
        return LinkedList { root: None, len: 0 };
    }
    fn prepend(&mut self, data: T) {
        let node = Box::new(Node {
            data: data,
            next: self.root.take(),
        });
        self.root.replace(node);
        self.len += 1;
    }
    fn append(&mut self, data: T) {
        let node = Box::new(Node {
            data: data,
            next: None,
        });

        if self.root.is_none() {
            self.root.replace(node);
            self.len += 1;
            return;
        }

        assert!(self.root.is_some());
        let mut tail_ref = self.root.as_mut().unwrap();
        loop {
            if tail_ref.next.is_none() {
                tail_ref.next.replace(node);
                self.len += 1;
                return;
            }
            tail_ref = tail_ref.next.as_mut().unwrap();
        }
    }

    fn first(&self) -> &T {
        assert!(self.root.is_some());
        let got = self.root.as_ref().unwrap().as_ref();
        let got = &got.data;
        return got;
    }
    fn len(&self) -> usize {
        return self.len;
    }
    fn foreach<F>(&self, mut callback: F)
    where
        F: FnMut(&T),
    {
        if self.root.is_none() {
            return;
        }
        let mut ptr = self.root.as_ref().unwrap().as_ref();
        loop {
            callback(&ptr.data);
            if ptr.next.is_none() {
                break;
            }
            ptr = ptr.next.as_ref().unwrap().as_ref();
        }
    }
    fn iter(&self) -> LinkedListIterator<T> {
        return LinkedListIterator { node: &self.root };
    }
}

struct LinkedListIterator<'a, T> {
    node: &'a Option<Box<Node<T>>>,
}

impl<'a, T> Iterator for LinkedListIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.node.is_none() {
            return None;
        }
        let to_ret = &self.node.as_ref().unwrap().as_ref().data;
        self.node = &self.node.as_ref().unwrap().next;
        return Some(to_ret);
    }
}

#[test]
fn test_prepend() {
    let mut ll = LinkedList::<i32>::new();
    ll.prepend(1);
    ll.prepend(2);
    assert_eq!(ll.len(), 2);
    assert_eq!(*ll.first(), 2);
}

#[test]
fn test_append() {
    let mut ll = LinkedList::<i32>::new();
    ll.append(1);
    ll.append(2);
    assert_eq!(ll.len(), 2);
    assert_eq!(*ll.first(), 1);
}

#[test]
fn test_foreach() {
    let mut ll = LinkedList::<i32>::new();
    ll.append(1);
    ll.append(2);
    let mut sum = 0;
    ll.foreach(|value| {
        sum += value;
    });
    assert_eq!(sum, 3);
}

#[test]
fn test_iterator() {
    let mut ll = LinkedList::<i32>::new();
    ll.append(1);
    ll.append(2);
    let mut it = ll.iter();
    assert_eq!(*it.next().unwrap(), 1);
    assert_eq!(*it.next().unwrap(), 2);
    assert_eq!(it.next(), None);
}

fn main() {
    println!("Hello, world!");

    let mut ll = LinkedList::<i32>::new();
    ll.append(1);
    ll.append(2);
    ll.append(3);
    println!("ll = {}", ll);
}
