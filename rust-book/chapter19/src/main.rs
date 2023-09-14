// Q: Why does the borrow checker not error?
// This creates two mutable references to a subslice.
// From The Rust Programming Language > 4.2:
// "At any given time, you can have either one mutable reference or any number of immutable references."
// A: My goof. See inline comment.
fn returns_mutable_subslice(x: &mut [i32]) -> &mut [i32] {
    return &mut x[0..1];
}

fn calls_returns_mutable_subslice() {
    let mut arr: [i32; 3] = [1, 2, 3];
    let m1 = returns_mutable_subslice(&mut arr);
    m1[0] = 2; // m1 is mutable reference
    println!("{:?}", arr);
    let m2 = returns_mutable_subslice(&mut arr);
    m2[0] = 3; // m2 is mutable reference
               // m1[0] = 4; When m1 is used again, receive this error:
               // "cannot borrow `arr` as mutable more than once at a time"
    println!("{:?}", arr);
}

extern "C" {
    fn abs(input: i32) -> i32;
}

fn calls_c_function() {
    unsafe {
        println!("abs(-2) = {}", abs(-2));
    }
}

struct Foo {
    v1: i32,
}

struct Bar {
    v2: i32,
}

impl std::ops::Add<Bar> for Foo {
    type Output = i32;
    fn add(self, other: Bar) -> i32 {
        return self.v1 + other.v2;
    }
}

trait MyTrait {
    fn myfn(&self);
}

struct Baz {}
impl MyTrait for Baz {
    fn myfn(&self) {
        println!("In Baz::myfn");
    }
}

fn fn_accepting_ref_to_trait(t: &dyn MyTrait) {
    t.myfn();
}

struct SomeType {
    x: i32,
}

struct NewType(SomeType);
use std::ops::Deref;

impl Deref for NewType {
    type Target = SomeType;
    fn deref(&self) -> &SomeType {
        return &self.0;
    }
}

fn callme(f: fn()) {
    f();
}

fn callback() {
    println!("called");
}

fn returns_callback() -> Box<dyn Fn(i32) -> i32> {
    let capture = 1;
    return Box::new(move |x| x + capture);
}

#[macro_export]
macro_rules! repeat3 {
    ( $x:expr ) => {
        $x;
        $x;
        $x;
    };
}

extern crate hello_macro;
use crate::hello_macro::HelloMacro;

extern crate hello_macro_derive;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct Pancakes {}

fn main() {
    calls_returns_mutable_subslice();
    calls_c_function();

    let f = Foo { v1: 1 };
    let b = Bar { v2: 2 };
    println!("f + b = {}", (f + b));

    let b2 = Baz {};
    b2.myfn();
    fn_accepting_ref_to_trait(&b2);

    let w = NewType {
        0: SomeType { x: 123 },
    };
    println!("w.0.x = {}", w.0.x);

    println!("w.x = {}", w.x); // expect not to compile.

    callme(callback);
    let cb = returns_callback();
    println!("cb(123) = {}", cb(123));

    repeat3!(println!("should be repeated 3 times"));

    Pancakes::hello_macro();
}
