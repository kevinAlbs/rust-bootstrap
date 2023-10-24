use core::fmt;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

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

    // Write SSTableInMemory on disk as follows:
    // [ key len as little-endian uint32 ] [ key ] [ value len as little endian uint32 ] [ value ]
    fn write_to_disk(&self, path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        let mut data = Vec::<u8>::new();
        for (k, v) in &self.strings {
            let klen = k.len() as u32;
            data.extend_from_slice(&klen.to_le_bytes());
            data.extend_from_slice(k.as_bytes());
            let vlen = v.len() as u32;
            data.extend_from_slice(&vlen.to_le_bytes());
            data.extend_from_slice(v.as_bytes());
        }
        // data.extend_from_slice()
        // Only call `fs::write` once to limit possible incomplete writes on process exit.
        std::fs::write(path, data);
        return Ok(());
    }
}

// LSMImpl implements thread-unsafe structures.
// LSMImpl is used by LSM by a lock.
struct LSMImpl {
    // When `active` reaches the maximum size, it is written to disk, and then reset.
    inmemory: SSTableInMemory,
}

impl LSMImpl {
    fn new() -> Self {
        return LSMImpl {
            inmemory: SSTableInMemory::new(),
        };
    }
}

// LSM may lose writes if the process exits.
// The `path` may not be used by more than one process.
// LSM is thread-safe.
#[derive(Clone)]
struct LSM {
    // `path` is the path to the directory containing the data for the LSM.
    path: String,
    lsmimpl: std::sync::Arc<std::sync::Mutex<LSMImpl>>,
}

impl LSM {
    fn new() -> Self {
        return LSM {
            path: "foo".to_string(),
            lsmimpl: std::sync::Arc::new(std::sync::Mutex::new(LSMImpl::new())),
        };
    }

    fn insert_str(&mut self, key: String, val: String) {
        println!("LSM inserting [{:?}] => {:?}", key, val);
        // TODO: lock.
        // TODO: if `inmemory` has capacity, insert.
        // TODO: else: write `inmemory` to disk.
        // TODO: check `inmemory`.
        // TODO: if `inmemory` does not contain `key`, check disk files.
        //
        return;
    }

    pub fn find_str(&self, key: String) -> Option<&String> {
        print!("LSM finding [{:?}]", key);
        return None;
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

struct TempFile {
    path: std::path::PathBuf,
}

impl TempFile {
    fn new(path: &std::path::Path) -> Self {
        return TempFile {
            path: path.to_path_buf(),
        };
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // Ignore error in case file does not exist. Test may have exited before creating the file.
        std::fs::remove_file(&self.path);
    }
}

#[test]
fn SSTableInMemory_can_write_to_disk() {
    let tempfile = TempFile::new(&std::path::Path::new(
        "SSTableInMemory_can_write_to_disk.db",
    ));
    let mut sst = SSTableInMemory::new();
    sst.insert_str("foo".to_string(), "bar".to_string());
    sst.write_to_disk(&tempfile.path)
        .expect("Should write to disk");
    // Read contents.
    let got = std::fs::read(&tempfile.path).expect("can read file");

    assert_eq!(
        got,
        vec![
            3, 0, 0, 0, //
            'f' as u8, 'o' as u8, 'o' as u8, //
            3, 0, 0, 0, //
            'b' as u8, 'a' as u8, 'r' as u8
        ]
    )
}

#[test]
fn LSM_can_insert_multithreaded() {
    let lsm = LSM::new();
    let mut lsm1 = lsm.clone();
    let handle1 = thread::spawn(move || {
        lsm1.insert_str(
            "key_from_thread1".to_string(),
            "value_from_thread1".to_string(),
        );
        lsm1.insert_str("shared_key".to_string(), "value_from_thread1".to_string());
    });

    let mut lsm2 = lsm.clone();
    let handle2 = thread::spawn(move || {
        lsm2.insert_str(
            "key_from_thread2".to_string(),
            "value_from_thread2".to_string(),
        );
        lsm2.insert_str("shared_key".to_string(), "value_from_thread2".to_string());
    });
    handle1.join().expect("handle1 should join");
    handle2.join().expect("handle2 should join");
    assert_eq!(
        lsm.find_str("shared_key".to_string()),
        Some(&"shared_value".to_string()),
    );
    let got = lsm
        .find_str("shared_key".to_string())
        .expect("should have value for `shared_key`");
    let got = got.to_owned();
    assert!(got == "value_from_thread1" || got == "value_from_thread2");
}
