// DONE: merge in-place
// DONE: merge with slice of i32
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

fn merge_inplace<T: std::cmp::PartialOrd + Copy> (s : &mut [T], lo1 : usize, hi1 : usize, lo2 : usize, hi2 : usize) {
    // lo1 and hi1 are inclusive.
    // lo2 and hi2 are inclusive.
    assert_eq!(hi1 + 1, lo2);

    // let mut out = [i32; (hi2 - lo1 + 1)]; // Cannot do. Array must be created with compile time const size.
    // create an output vector.
    let mut out = Vec::with_capacity (hi2 - lo1 + 1);
    let mut i1 = lo1;
    let mut i2 = lo2;
    loop {
        let min;
        if i1 == hi1 + 1 && i2 == hi2 + 1 {
            break;
        }
        if i1 == hi1 + 1 {
            min = s[i2];
            i2 += 1;
        } else if i2 == hi2 + 1 {
            min = s[i1];
            i1 += 1;
        } else {
            if s[i1] < s[i2] {
                min = s[i1];
                i1 += 1;
            } else {
                min = s[i2];
                i2 += 1;
            }
        }
        out.push(min)
    }
    for (idx, val) in out.iter().enumerate() {
        s[idx] = *val;
    }
}

fn mergesort (s : &mut [i32]) {

}


#[test]
fn test_merge () {
    assert_eq!(merge_separate(vec![1], vec![0]), vec![0,1]);
    // Can merge in place.
    {
        let mut s = [1,3,5,2,4,6];
        merge_inplace (&mut s, 0, 2, 3, 5);
        assert_eq!(s, [1,2,3,4,5,6]);
    }
    // Can merge in place with two elements.
    {
        let mut s = [2, 1];
        merge_inplace (&mut s, 0, 0, 1, 1);
        assert_eq!(s, [1,2]);
    }
    // Can merge floats.
    {
        let mut s = [1.0,3.0,5.0,2.0,4.0,6.0];
        merge_inplace (&mut s, 0, 2, 3, 5);
        assert_eq!(s, [1.0,2.0,3.0,4.0,5.0,6.0]);
    }
    // Can merge sort.
    {
        let mut s = [3,1,2];
        mergesort (&mut s);
        assert_eq!(s, [1,2,3]);
    }
}