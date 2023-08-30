// DONE: read https://doc.rust-lang.org/std/cell/index.html
// DONE: make failing test with multiple threads.
// DONE: implement `clone()`
// N/A?: implement Sync trait.
//   > This trait is automatically implemented when the compiler determines it’s appropriate.
// DONE: make `enqueue` not require a mutable reference.
use std::sync::Arc;
use std::sync::Mutex;

struct TSQueue<T> {
    data: Arc<Mutex<Vec<T>>>,
}

impl<T> TSQueue<T> {
    pub fn new() -> Self {
        return TSQueue {
            data: Arc::new(Mutex::new(Vec::<T>::new())),
        };
    }

    pub fn enqueue(&self, value: T) {
        let mut data = self.data.lock().expect("should lock");
        (*data).push(value);
    }

    pub fn dequeue(&self) -> Option<T> {
        let mut data = self.data.lock().expect("should lock");

        if (*data).len() == 0 {
            return None;
        }
        return Some((*data).remove(0));
    }

    pub fn len(&self) -> usize {
        return 0;
    }

    pub fn clone(&self) -> TSQueue<T> {
        return TSQueue {
            data: self.data.clone(),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn works_with_single_thread() {
        let tsq = TSQueue::<i32>::new();
        tsq.enqueue(123);
        tsq.enqueue(456);
        assert_eq!(tsq.dequeue(), Some(123));
        assert_eq!(tsq.dequeue(), Some(456));
        assert_eq!(tsq.dequeue(), None);
    }

    fn works_with_many_threads() {
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
    }
}
