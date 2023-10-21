use core::fmt;
use std::collections::HashMap;
use std::error::Error;

struct SSTableOnDisk {}

struct SSTableInMemory {
    // Reserve maximum size of in memory store.
    // map: HashMap<[u8], [u8]>
    data: Vec<u8>,
    storage: HashMap<Vec<u8>, Vec<u8>>,
    strings: HashMap<String, String>,
    size: usize,
}

#[derive(Debug)]
struct SSTableHasNoCapacityError {}
impl Error for SSTableHasNoCapacityError {}
impl fmt::Display for SSTableHasNoCapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SSTable does not have capacity",)
    }
}

impl SSTableInMemory {
    const MAX_SIZE: usize = 4096;
    fn new() -> Self {
        return SSTableInMemory {
            // map: HashMap
            data: Vec::<u8>::with_capacity(Self::MAX_SIZE),
            storage: HashMap::new(),
            strings: HashMap::new(),
            size: 0,
        };
    }

    fn insert_str(&mut self, key: String, val: String) -> Option<SSTableHasNoCapacityError> {
        println!("inserting [{:?}] => {:?}", key, val);
        // Check for capacity;
        let mut new_size = self.size;
        {
            if self.strings.contains_key(&key) {
                // Subtract first.
                new_size -= key.len();
                new_size -= self.strings.get(&key).unwrap().len();
            }
            new_size += key.len();
            new_size += val.len();
            if new_size > Self::MAX_SIZE {
                return Some(SSTableHasNoCapacityError {});
            }
        }
        self.strings.insert(key, val);
        self.size = new_size;
        return None;
    }

    // If found, returns a copy of the value.
    // Q: Why does return type not require a lifetime?
    // A:
    pub fn find_str(&self, key: String) -> Option<&String> {
        print!("finding [{:?}]", key);
        return self.strings.get(&key);
    }

    fn write_to_disk() -> Result<(), Box<dyn Error>> {
        todo!("Not yet implemented");
        return Ok(());
    }
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn SSTableInMemory_can_insert_and_find() {
    let mut sst = SSTableInMemory::new();
    let got = sst.insert_str("foo".to_string(), "bar".to_string());
    assert!(got.is_none());
    let got = sst.find_str("foo".to_string());
    assert_eq!(got, Some(&"bar".to_string()));
}

#[test]
fn SSTableInMemory_tracks_size() {
    let mut sst = SSTableInMemory::new();
    let large_str = "a".to_string().repeat(SSTableInMemory::MAX_SIZE - 1);
    {
        // Insert a key with 1 byte, and a value with MAX_SIZE - 1 bytes.
        let got = sst.insert_str("a".to_string(), large_str.clone());
        assert!(got.is_none());
        // Expect an error on the subsequent insert.
        let got = sst.insert_str("b".to_string(), "c".to_string());
        assert!(got.is_some());
    }

    // Test overwriting existing values.
    {
        // Insert a key with 1 byte, and a value with MAX_SIZE - 1 bytes.
        let got = sst.insert_str("a".to_string(), large_str);
        assert!(got.is_none());
        // Expect no error on overwrite.
        let got = sst.insert_str("a".to_string(), "b".to_string());
        assert!(got.is_none());
    }
}
