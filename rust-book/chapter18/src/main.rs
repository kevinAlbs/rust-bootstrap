struct Point {
    x: i32,
    y: i32
}

fn main() {
    let x = 5;

    match x {
        5 => println!("5"),
        6 => println!("6"),
        _ => println!("Default case, x = {:?}", x),
    }

    let p = Point{x: 1, y: 2};
    match p {
        Point{x,y:3} => println!("where y == 3, x={x}"),
        Point{x,y} => println!("default: x={x}, y={y}"),
    }

    let op = Option::<Point>::Some(Point{x:1,y:2});
    match op {
        Some(_) => println!("op is Some something"),
        None => println!("op is None"),
    };

    let tup = (1,8,3,4, Point{x:1,y:2});
    match tup {
        (1,2,..) => println!("tup starts with 1,2"),
        (1,3,..) => println!("tup starts with 1,3"),
        (..,Point{x:_, y}) if y % 2 == 0 => println!("tup ends with Point with even y"),
        _ => println!("tup is something else")
    };

    let id = 5;
    match id {
        3..=7 
    }
}
