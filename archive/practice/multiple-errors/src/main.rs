use std::error::Error;
use std::result::Result;

fn something_returning_multiple_errors(input: &str, input2: &str) -> Result<(), Box<dyn Error>> {
    // let res: Result<i32, std::num::ParseIntError> = input.parse();
    // if res.is_err() {
    //     return Result::Err(Box::new(res.err().unwrap()));
    // }
    // let as_i32: i32 = res.unwrap();

    let as_i32: i32 = input.parse()?;

    let res: Result<f64, std::num::ParseFloatError> = input.parse();
    if res.is_err() {
        return Err(From::from(res.err().unwrap()));
    }
    let as_f64: f64 = input2.parse().unwrap();
    println!("fn got i32={}, and f64={}", as_i32, as_f64);
    return Result::Ok(());
}
fn main() {
    let res = something_returning_multiple_errors("123", "4.5");
    if res.is_err() {
        println!("main got error: {:?}", res);
    } else {
        println!("main got OK: {:?}", res);
    }
}
