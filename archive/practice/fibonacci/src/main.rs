// A generic Fibonacci.
// The Fibonacci sequence adds: 1, 2, 3, 5, ...
// The generic Fibonacci sequence supports other binary operations.

trait BinaryOp<T = i32> {
    fn compute(&self, a: T, b: T) -> T;
}

struct Adder {}
impl BinaryOp for Adder {
    fn compute(&self, a: i32, b: i32) -> i32 {
        return a + b;
    }
}

struct Subtractor {}
impl BinaryOp for Subtractor {
    fn compute(&self, a: i32, b: i32) -> i32 {
        return a - b;
    }
}

fn fib(n: usize) -> i32 {
    let mut lhs = 0;
    let mut rhs = 1;
    let mut counter: usize = n;
    while counter > 0 {
        let res = lhs + rhs;
        lhs = rhs;
        rhs = res;
        counter -= 1;
    }
    return rhs;
}

fn fib_with_ops(ops: Vec<&dyn BinaryOp>) -> i32 {
    let mut lhs = 0;
    let mut rhs = 1;
    for op in ops {
        let res = op.compute(lhs, rhs);
        lhs = rhs;
        rhs = res;
    }
    return rhs;
}

fn fib_with_ops_generic<T: BinaryOp>(ops: Vec<&T>) -> i32 {
    let mut lhs = 0;
    let mut rhs = 1;
    for op in ops {
        let res = op.compute(lhs, rhs);
        lhs = rhs;
        rhs = res;
    }
    return rhs;
}

#[test]
fn test_fib() {
    assert_eq!(fib(0), 1);
    assert_eq!(fib(1), 1);
    assert_eq!(fib(2), 2);
    assert_eq!(fib(3), 3);
    assert_eq!(fib(4), 5);
}

#[test]
fn test_fib_with_ops() {
    assert_eq!(fib_with_ops(vec![]), 1);
    assert_eq!(fib_with_ops(vec![&Adder {}]), 1);
    assert_eq!(fib_with_ops(vec![&Adder {}, &Adder {}]), 2);
    assert_eq!(fib_with_ops(vec![&Adder {}, &Adder {}, &Adder {}]), 3);
    // fib_with_opts can support other objects that implement BinaryOp.
    assert_eq!(fib_with_ops(vec![&Adder {}, &Subtractor {}]), 0);
}

#[test]
fn test_fib_with_ops_generic() {
    assert_eq!(fib_with_ops_generic::<Adder>(vec![]), 1);
    assert_eq!(fib_with_ops_generic::<Adder>(vec![&Adder {}]), 1);
    assert_eq!(fib_with_ops_generic::<Adder>(vec![&Adder {}, &Adder {}]), 2);
    // Generic version cannot support different types that implement BinaryOp.
}

fn main() {
    println!("Hello, world!");
}
