// Status: Incomplete
// Library to calculate percentiles in a background thread.
// TODO: Make the numeric type generic.
// TODO: Return a Result in `get_percentile`. Return an error if data is empty.
// Q: What happens when `as` is used with out-of-range value?
// A: Casting f64 to i32 appears to cap value:
// ```
// let as_f64 = 4294967297.0;
// let as_i32 = as_f64 as i32;
// println! ("as_i32={:?}, as_f64={:?}", as_i32, as_f64);
// // Prints:
// // as_i32=2147483647, as_f64=4294967297.0
// ```

pub struct PercentileCalculator {
    data : Vec<f64>
}

impl PercentileCalculator {
    pub fn new () -> Self {
        return Self {
            data: Vec::new()
        };
    }

    pub fn add (&mut self, entry : f64) {
        self.data.push(entry);
        // f64 does not implement `Ord` trait because NaN != NaN.
        // Assume data does not contain `NaN` and use `partial_cmp`.
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    pub fn get_percentile(&self, p : f64) -> f64 {
        let mut last = self.data.len() - 1;
        if last < 0 {
            return 0.0;
        }

        let percent;
        if p < 0.0 {
            percent = 0.0;
        } else if p > 100.0 {
            percent = 100.0;
        } else {
            percent = p;
        }
        let target = (last as f64 * (percent / 100.0)) as usize;
        return self.data[target];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        {
            let mut pc = PercentileCalculator::new();
            for i in 0..10 {
                // Q: How to cast from i32 to f64?
                // A: Use `as` keyword.
                pc.add (1.0 + i as f64);
            }
            assert_eq!(pc.get_percentile(10.0), 1.0);
            assert_eq!(pc.get_percentile(100.0), 10.0);
        }
        // Test adding data in descending order.
        {
            let mut pc = PercentileCalculator::new();
            for i in 0..10 {
                pc.add (10.0 - i as f64);
            }
            assert_eq!(pc.get_percentile(10.0), 1.0);
            assert_eq!(pc.get_percentile(100.0), 10.0);
        }
    }
}
