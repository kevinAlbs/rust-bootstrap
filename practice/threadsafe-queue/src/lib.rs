// DONE: read https://doc.rust-lang.org/std/cell/index.html
// DONE: make failing test with multiple threads.
// TODO: implement `clone()`
// TODO: implement Sync trait.
// TODO: make `enqueue` not require a mutable reference.
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::{collections::VecDeque, marker::PhantomData};

struct TSQueue<T> {
    phantom: PhantomData<T>,
    data: Vec<T>,
}

impl<T> TSQueue<T> {
    pub fn new() -> Self {
        return TSQueue {
            phantom: PhantomData::<T> {},
            data: Vec::<T>::new(),
        };
    }

    pub fn enqueue(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.data.len() == 0 {
            return None;
        }
        return Some(self.data.remove(0));
    }

    pub fn len(&self) -> usize {
        return 0;
    }
}

unsafe impl<T> Send for TSQueue<T> {}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn works_with_single_thread() {
        let mut tsq = TSQueue::<i32>::new();
        tsq.enqueue(123);
        tsq.enqueue(456);
        assert_eq!(tsq.dequeue(), Some(123));
        assert_eq!(tsq.dequeue(), Some(456));
        assert_eq!(tsq.dequeue(), None);
    }

    fn works_with_many_threads() {
        // let mutable_tsq = Arc::new(Mutex::new(TSQueue::<i32>::new()));

        // let mt1 = mutable_tsq.clone();
        // let t1 = thread::spawn(move || {
        //     mt1.lock().expect("can lock").enqueue(123);
        // });
        // let mt2 = mutable_tsq.clone();
        // let t2 = thread::spawn(move || {
        //     mt2.lock().expect("can lock").enqueue(456);
        // });
        // t1.join().expect("should have joined");
        // t2.join().expect("should have joined");

        let tsq = TSQueue::<i32>::new();
        let tsq_ref1 = tsq.clone();
        let t1 = thread::spawn(move || {
            tsq_ref1.enqueue(123);
        });
        let tsq_ref2 = tsq.clone();
        let t2 = thread::spawn(move || {
            tsq_ref2.enqueue(123);
        });
        t1.join().expect("should have joined");
        t2.join().expect("should have joined");

        // static foo: i32 = 123;
        // let refFoo: &i32 = &foo;

        // // Q: Can refFoo be 'moved' to multiple threads?
        // // A: Yes?
        // let t3 = thread::spawn(move || {
        //     println!("got: {}", refFoo);
        // });
        // let t4 = thread::spawn(move || {
        //     println!("got: {}", refFoo);
        // });

        // t3.join().expect("should have joined");
        // t4.join().expect("should have joined");
    }
}
