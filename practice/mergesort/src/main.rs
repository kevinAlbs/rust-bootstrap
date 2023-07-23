// TODO: merge in-place
// TODO: merge with slice of i32
// TODO: make merge generic
// TODO: implement sort

fn main() {
    println!("Hello, world!");
}

// = help: the trait `Sized` is not implemented for `[i32]`
// = note: the return type of a function must have a statically known size
fn merge_separate(a1 : Vec<i32>, a2  : Vec<i32>) -> Vec<i32>
{
    let mut ret = Vec::new();
    let mut i1 = 0;
    let mut i2 = 0;
    loop {
        let min;
        if i1 == a1.len() && i2 == a2.len() {
            break;
        }
        if i1 == a1.len() {
            min = a2[i2];
            i2 += 1;
        } else if i2 == a2.len() {
            min = a1[i1];
            i1 += 1;
        } else {
            if a1[i1] < a2[i2] {
                min = a1[i1];
                i1 += 1;
            } else {
                min = a2[i2];
                i2 += 1;
            }
        }
        ret.push(min)
    }
    return ret;
}

fn merge_inplace(s : &mut [i32], lo1 : usize, hi1 : usize, lo2 : usize, hi2 : usize) {}


#[test]
fn test_merge () {
    assert_eq!(merge_separate(vec![1], vec![0]), vec![0,1]);
    let mut s = [1,3,5,2,4,6];
    merge_inplace (&mut s, 0, 2, 3, 6);
    assert_eq!(s, [1,2,3,4,5,6]);
}