use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

fn main() {
    println!("Hello, world!");
    let mut foo = 123;
    let fooImm = &foo;
    let fooMut = &mut foo;
    fooMut += 1;
    // fooImm has changed!

    let arcToFoo = Arc::new(Mutex::new(123));

    let arcToFooClone = arcToFoo.clone();

    let t1 = thread::spawn(move || {
        println!("foo={}", arcToFooClone.lock().expect("should lock"));
    });

    // let t2 = thread::spawn(|| {
    //     *fooMut += 1;
    // });

    t1.join();
    // t2.join();
}
