fn main() {
    let gifts = [
        "Twelve drummers drumming",
        "Eleven pipers piping    ",
        "Ten lords a-leaping ",
        "Nine ladies dancing ",
        "Eight maids a-milking   ",
        "Seven swans a-swimming  ",
        "Six geese a-laying  ",
        "Five gold rings (five golden rings) ",
        "Four calling birds  ",
        "Three French hens   ",
        "Two turtledoves ",
        "And a partridge in a pear tree  ",
    ];
    let day_to_str = [
        "first",
        "second",
        "third",
        "fourth",
        "fifth",
        "sixth",
        "seventh",
        "eight",
        "ninth",
        "tenth",
        "eleventh",
        "twelfth"
    ];
    for i in 0..12 {
        let nth = day_to_str[i];
        println!("On the {nth} day of Christmas, my true love gave to me");
        if i == 0 {
            println! ("A partridge in a pear tree");
            continue;
        }
        for i in (0..i+1).rev() {
            let lyric = gifts[12 - i - 1];
            println!("{lyric}");
        }
        println!();
    }
}
