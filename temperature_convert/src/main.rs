// Convert temperatures between Fahrenheit and Celsius.

use std::io;
fn main() {
    println!("Enter unit: (C/F):");
    let mut unit = String::new();
    let res = io::stdin().read_line(&mut unit);
    match res {
        Ok(_) => {}
        Err(error) => {
            println!("failed to read unit: {:?}", error);
            std::process::exit(1);
        }
    };
    unit = unit.trim().to_string();
    if !unit.eq("C") && !unit.eq("F") {
        println!("expected C or F, got {unit}");
        std::process::exit(1);
    }

    println!("Enter temperature:");
    let mut temperature = String::new();
    let res = io::stdin().read_line(&mut temperature);
    match res {
        Ok(_) => {}
        Err(error) => {
            println!("failed to read temperature: {:?}", error);
            std::process::exit(1);
        }
    };
    temperature = temperature.trim().to_string();
    let res = temperature.parse::<f64>();
    let temperature = match res {
        Ok(val) => val,
        Err(error) => {
            println!("failed to parse {temperature} as float64: {:?}", error);
            std::process::exit(1);
        }
    };
    println!("Entered temperature {temperature} {unit}");
    if unit == "C" {
        let converted = temperature * 1.8 + 32.;
        println!("Converted to: {converted} F");
    } else if unit == "F" {
        let converted = (temperature - 32.) / 1.8;
        println!("Converted to: {converted} C");
    } else {
        unreachable!("Expecting C or F");
    }
}
