fn main() {
    // // Use of ? in Result types:
    // {
    //     type ParseResult = Result<i32, std::num::ParseIntError>;
    //     fn multiply(a: &str, b: &str) -> ParseResult {
    //         let ai32: ParseResult = a.parse::<i32>();
    //         let bi32: ParseResult = b.parse::<i32>();
    //         Ok(ai32? * bi32?)
    //     }

    //     let args: Vec<String> = std::env::args().collect();
    //     if args.len() < 3 {
    //         println!("Usage: {} <num1> <num2>", args[0]);
    //         return;
    //     }
    //     let res = multiply(args[1].as_str(), args[2].as_str());
    //     match res {
    //         Ok(val) => println!("Result: {}", val),
    //         Err(e) => println!("Error: {}", e),
    //     }
    // }

    // Define custom error type.
    // {
    //     use std::fmt;
    //     type Result<T> = std::result::Result<T, DoubleError>;
    //     #[derive(Clone, Debug)]
    //     struct DoubleError;

    //     impl fmt::Display for DoubleError {
    //         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //             write!(f, "got DoubleError")?;
    //             return Ok(());
    //         }
    //     }

    //     fn double_first(vec: Vec<String>) -> Result<i32> {
    //         let first = vec.first().ok_or(DoubleError {})?;
    //         let first_i32: i32 = first.parse().map_err(|_| DoubleError {})?;
    //         return Ok(first_i32 * 2);
    //     }

    //     let args: Vec<String> = std::env::args().collect();
    // }

    // // Box an Error. The stdlib implements a conversion from any type implementing the Error trait to Box<Error> via From.
    // {
    //     use std::error::Error;

    //     #[derive(Debug)]
    //     struct DevilishError {}
    //     impl std::fmt::Display for DevilishError {
    //         fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    //             write!(f, "DevilishError encountered.")?;
    //             return Ok(());
    //         }
    //     }
    //     impl Error for DevilishError {}

    //     fn add_one(numeric: String) -> Result<(), Box<dyn Error>> {
    //         let got = numeric.parse::<i32>()?;
    //         if got == 666 {
    //             return Err(Box::new(DevilishError {}));
    //         }
    //         return Ok(());
    //     }

    //     let got = add_one("666".to_string());
    //     if got.is_err() {
    //         let err = got.err().unwrap();
    //         if err.is::<DevilishError>() {
    //             println!("Got DevilishError");
    //             let d = err.downcast_ref::<DevilishError>().unwrap();
    //             println!("Downcasted error: {}", d);
    //         }
    //         println!("err = {}", err);
    //     }
    // }

    // // Wrap an error.
    // {
    //     #[derive(Debug)]
    //     struct AddError {
    //         msg: String,
    //         src: Option<Box<dyn std::error::Error>>,
    //     }

    //     impl AddError {
    //         fn new(msg: String) -> Self {
    //             return AddError { msg, src: None };
    //         }
    //         fn with(msg: String, src: Box<dyn std::error::Error>) -> Self {
    //             return AddError {
    //                 msg: msg,
    //                 src: Some(src),
    //             };
    //         }
    //     }

    //     impl std::fmt::Display for AddError {
    //         fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    //             write!(f, "AddError with msg: {}", self.msg)?;
    //             if self.src.is_some() {
    //                 write!(f, ". Source: {}", self.src.as_ref().unwrap())?;
    //             }
    //             return Ok(());
    //         }
    //     }

    //     fn increment_u8(numeric: String) -> Result<u8, AddError> {
    //         let got = numeric.parse::<u8>();
    //         if got.is_err() {
    //             return Err(AddError::with(
    //                 "Could not parse".to_string(),
    //                 Box::new(got.err().unwrap()),
    //             ));
    //         }
    //         let got = got.unwrap();
    //         if got == u8::MAX {
    //             return Err(AddError::new("Cannot add more".to_string()));
    //         }
    //         return Ok(got + 1);
    //     }

    //     // No error.
    //     let g1 = increment_u8("1".to_string());
    //     assert!(g1.is_ok());
    //     assert_eq!(g1.unwrap(), 2);

    //     // Max error.
    //     let g1 = increment_u8("255".to_string());
    //     assert!(g1.is_err());
    //     println!("Input {} got error: {}", "255", g1.err().unwrap());

    //     // Parse error.
    //     let g1 = increment_u8("foo".to_string());
    //     assert!(g1.is_err());
    //     println!("Input {} got error: {}", "foo", g1.err().unwrap());
    // }

    {
        let v = vec!["1", "2", "3", "foo", "bar"];
        let res: Result<Vec<_>, _> = v.into_iter().map(|s| s.parse::<i32>()).collect();
        println!("res={:?}", res);
    }
}
