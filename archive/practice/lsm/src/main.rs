use core::fmt;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

// Q: What does SSTable stand for?
// A: Sorted Strings Table.
struct SSTableInMemory {
    // Reserve maximum size of in memory store.
    // map: HashMap<[u8], [u8]>
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
            strings: HashMap::new(),
            size: 0,
        };
    }

    fn insert_str(&mut self, key: String, val: String) -> Result<(), Box<dyn Error>> {
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
                return Err(Box::new(SSTableHasNoCapacityError {}));
            }
        }
        self.strings.insert(key, val);
        self.size = new_size;
        return Ok(());
    }

    // If found, returns a copy of the value.
    // Q: Why does return type not require a lifetime?
    // A:
    pub fn find_str(&self, key: String) -> Option<&String> {
        return self.strings.get(&key);
    }

    // Write SSTableInMemory on disk as follows:
    // [ key len as little-endian uint32 ] [ key ] [ value len as little endian uint32 ] [ value ]
    fn write_to_disk(&self, path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        let mut data = Vec::<u8>::new();

        // Sort list.
        let mut keys_vec = Vec::<String>::new();
        for key in self.strings.keys() {
            keys_vec.push(key.clone());
        }
        keys_vec.sort();

        for k in keys_vec {
            let v = self.strings.get(&k).expect("should have key");
            let klen = k.len() as u32;
            data.extend_from_slice(&klen.to_le_bytes());
            data.extend_from_slice(k.as_bytes());
            let vlen = v.len() as u32;
            data.extend_from_slice(&vlen.to_le_bytes());
            data.extend_from_slice(v.as_bytes());
        }
        // Only call `fs::write` once to limit possible incomplete writes on process exit.
        // Q: Can the error returned by `write` be annotated with the path?
        // When the directory did not exist, this error was returned:
        // "No such file or directory (os error 2)"
        // It may be easier to identify which operation errored if the path is added.
        // A:
        std::fs::write(path, data)?;
        return Ok(());
    }

    fn clear(&mut self) {
        self.size = 0;
        self.strings.clear();
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
    // `datapath` is the path to the directory containing the data for the LSM.
    // The data directory.
    // Data files are protected by lsmimpl Mutex.
    datapath: PathBuf,
    lsmimpl: Arc<Mutex<LSMImpl>>,
}

impl LSM {
    fn new() -> Self {
        return LSM {
            datapath: PathBuf::from("data"),
            lsmimpl: Arc::new(Mutex::new(LSMImpl::new())),
        };
    }

    // Caller must hold Mutex for lsmimpl.
    fn read_count(&self) -> Result<i32, Box<dyn Error + '_>> {
        let datapath = std::path::PathBuf::from(&self.datapath);
        let countpath = datapath.join("count.txt");
        if !countpath.exists() {
            return Ok(0);
        }
        let contents_vec = std::fs::read(countpath.clone())?;
        let contents_str = String::from_utf8(contents_vec)?;
        let count: i32 = contents_str.parse()?;
        return Ok(count);
    }

    // Caller must hold Mutex for lsmimpl.
    fn write_count(&self, count: i32) -> Result<(), Box<dyn Error + '_>> {
        let datapath = std::path::PathBuf::from(&self.datapath);
        let countpath = datapath.join("count.txt");
        fs::write(countpath, format!("{}", count))?;
        return Ok(());
    }

    // Use `+ '_` to silence lifetime error. Assume the returned Error lives as long as `self`.
    // Q: Why is the '_ needed for the lifetime? A:
    fn insert_str(&mut self, key: String, val: String) -> Result<(), Box<dyn Error + '_>> {
        println!("LSM inserting key={:?}", key);
        let mut lsm = self.lsmimpl.lock()?;
        let res = lsm.inmemory.insert_str(key.clone(), val.clone());
        if !res.is_err() {
            // Insert succeeded.
            return Ok(());
        }
        // Error occurred. Was it due to no capacity?
        let err = res.err().unwrap();
        if !err.is::<SSTableHasNoCapacityError>() {
            // Unhandled error. Propagate.
            return Err(err);
        }
        // Try to write `inmemory` to disk.
        {
            // Try to read count.
            let count = self.read_count()?;
            let datapath = std::path::PathBuf::from(&self.datapath);
            let filepath = datapath.join(format!("{:04}.dat", count));
            lsm.inmemory.write_to_disk(&filepath)?;

            // Write new count.
            self.write_count(count + 1)?;
        }

        // Flush `inmemory`.
        lsm.inmemory.clear();
        return lsm.inmemory.insert_str(key.clone(), val.clone());
    }

    // `flush` is a test convenience to force flushing the in-memory SSTable to disk.
    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        todo!("Not yet implemented");
    }

    pub fn find_str(&self, key: String) -> Result<Option<String>, Box<dyn Error + '_>> {
        println!("LSM finding [{:?}]", key);
        let lsm = self.lsmimpl.lock()?;
        let res = lsm.inmemory.find_str(key.clone());
        if res.is_some() {
            return Ok(Some(res.unwrap().to_owned()));
        }
        // If `inmemory` does not contain `key`, check disk files.

        // TODO: for simplicity, scan disk file linearly.
        let count = self.read_count()?;
        println!("read count: {}", count);
        if count == 0 {
            return Ok(None);
        }

        for file_idx in (0..count).rev() {
            println!("checking file {}", file_idx);
            let datapath = std::path::PathBuf::from(&self.datapath);
            let filepath = datapath.join(format!("{:04}.dat", file_idx));
            let mut f = fs::OpenOptions::new().read(true).open(filepath.clone())?;

            // TODO: Iterate over file.
            loop {
                // Read key length.
                let mut key_len_buf: [u8; 4] = [0; 4];
                // TODO: break on unexpected EOF.
                let res = f.read_exact(&mut key_len_buf);
                if res.is_err() {
                    let err = res.err().unwrap();
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        // Reached end of file.
                        break;
                    }
                }

                let key_len: u32 = u32::from_le_bytes(key_len_buf);
                println!("read key_len: {}", key_len);
                // Read key.
                let mut key_buf = Vec::<u8>::new();
                key_buf.resize(key_len as usize, 0);
                f.read_exact(key_buf.as_mut_slice())?;
                let key_str = String::from_utf8(key_buf)?;
                println!("read key: {}", key_str);

                // Read value length.
                let mut value_len_buf: [u8; 4] = [0; 4];
                f.read_exact(&mut value_len_buf)?;
                let value_len: u32 = u32::from_le_bytes(value_len_buf);
                println!("read value_len: {}", value_len);

                if key_str == key {
                    // Found match. Read and return value.
                    let mut value_buf = Vec::<u8>::new();
                    value_buf.resize(value_len as usize, 0);
                    f.read_exact(value_buf.as_mut_slice())?;
                    let value_str = String::from_utf8(value_buf)?;
                    return Ok(Some(value_str));
                }

                // Did not find match. Skip value.
                f.seek(std::io::SeekFrom::Current(value_len as i64))?;
            }
        }

        return Ok(None);
    }

    fn read_key(&self, f: &mut std::fs::File) -> Result<Option<String>, Box<dyn Error + '_>> {
        // Read key length.
        let mut key_len_buf: [u8; 4] = [0; 4];
        let res = f.read_exact(&mut key_len_buf);
        if res.is_err() {
            let err = res.err().unwrap();
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                // Reached end of file.
                return Ok(None);
            }
            return Err(Box::new(err));
        }

        let key_len: u32 = u32::from_le_bytes(key_len_buf);
        println!("read key_len: {}", key_len);
        // Read key.
        let mut key_buf = Vec::<u8>::new();
        key_buf.resize(key_len as usize, 0);
        f.read_exact(key_buf.as_mut_slice())?;
        let key_str = String::from_utf8(key_buf)?;
        println!("read key: {}", key_str);
        return Ok(Some(key_str));
    }

    fn read_value(&self, f: &mut std::fs::File) -> Result<Option<String>, Box<dyn Error + '_>> {
        // Read value length.
        let mut value_len_buf: [u8; 4] = [0; 4];
        let res = f.read_exact(&mut value_len_buf);
        if res.is_err() {
            let err = res.err().unwrap();
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                // Reached end of file.
                return Ok(None);
            }
            return Err(Box::new(err));
        }

        let value_len: u32 = u32::from_le_bytes(value_len_buf);
        println!("read value_len: {}", value_len);
        // Read value.
        let mut value_buf = Vec::<u8>::new();
        value_buf.resize(value_len as usize, 0);
        f.read_exact(value_buf.as_mut_slice())?;
        let value_str = String::from_utf8(value_buf)?;
        println!("read value: {}...", value_str.get(0..3).unwrap());
        return Ok(Some(value_str));
    }

    fn write_key_value(
        &self,
        f: &mut std::fs::File,
        k: &String,
        v: &String,
    ) -> Result<(), Box<dyn Error + '_>> {
        println!("writing key {} and value {}...", k, v.get(0..3).unwrap());
        let mut data = Vec::<u8>::new();
        let klen = k.len() as u32;
        data.extend_from_slice(&klen.to_le_bytes());
        data.extend_from_slice(k.as_bytes());
        let vlen = v.len() as u32;
        data.extend_from_slice(&vlen.to_le_bytes());
        data.extend_from_slice(v.as_bytes());
        f.write_all(data.as_slice())?;
        return Ok(());
    }

    fn merge(&mut self) -> Result<(), Box<dyn Error + '_>> {
        // TODO: hold lock. Read count. Merge files <= count.
        let lsm = self.lsmimpl.lock().expect("should lock");
        let datapath = std::path::PathBuf::from(&self.datapath);

        // Read count.
        let count = self.read_count()?;

        let mut idx_to_file = HashMap::<i32, std::fs::File>::new();
        let mut idx_to_last_key = HashMap::<i32, Option<String>>::new();
        for file_idx in 0..count {
            let filepath = datapath.join(format!("{:04}.dat", file_idx));
            let f = fs::OpenOptions::new().read(true).open(filepath.clone())?;
            let got = idx_to_file.insert(file_idx, f);
            assert!(got.is_none()); // Should not have overwritten a value.
            let first_key = self.read_key(idx_to_file.get_mut(&file_idx).unwrap())?;
            let got = idx_to_last_key.insert(file_idx, first_key);
            assert!(got.is_none()); // Should not have overwritten a value.
        }

        println!("read the first key of each file: {:?}", idx_to_last_key);

        println!("merge ... begin");

        // Delete merged.dat if it exists.
        let outfile_path = datapath.join("merged.dat");
        if outfile_path.exists() {
            std::fs::remove_file(outfile_path)?;
        }

        {
            // Merge into one file.
            let mut outfile = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(datapath.join("merged.dat"))
                .expect("should open merged.dat");

            loop {
                // Get the smallest key of all files.
                let mut smallest_key: Option<String> = None;
                let mut last_idx_with_smallest_key: Option<i32> = None;
                // TODO: use a heap to optimize "get the smallest key".
                for file_idx in 0..count {
                    let key = idx_to_last_key.get(&file_idx).expect("should have entry");
                    if key.is_none() {
                        continue;
                    }
                    let key = key.as_ref().unwrap();

                    if smallest_key.is_none() {
                        // Set initial smallest key.
                        smallest_key = Some(key.clone());
                        last_idx_with_smallest_key = Some(file_idx);
                        continue;
                    }

                    if *key < *smallest_key.as_ref().unwrap() {
                        // Set new smallest key.
                        smallest_key = Some(key.clone());
                        last_idx_with_smallest_key = Some(file_idx);
                        continue;
                    }

                    if *key == *smallest_key.as_ref().unwrap() {
                        // Another entry for smallest key found. Set last index.
                        last_idx_with_smallest_key = Some(file_idx);
                        continue;
                    }
                }

                if smallest_key.is_none() {
                    println!("merge ... read all keys. Breaking.");
                    break;
                }
                println!(
                    "merge ... got smallest key {} from file {:04}.dat",
                    smallest_key.as_ref().unwrap(),
                    last_idx_with_smallest_key.unwrap()
                );

                // Write value from the file with the largest index containing `smallest_key`.
                {
                    let idx = last_idx_with_smallest_key.unwrap();
                    let f = idx_to_file.get_mut(&(idx.clone())).unwrap();
                    let value = self.read_value(f).expect("should have value").unwrap();
                    self.write_key_value(&mut outfile, smallest_key.as_ref().unwrap(), &value)?;
                    // Read next key.
                    let next_key = self.read_key(f)?;
                    idx_to_last_key.insert(idx, next_key);
                }

                // Discard values for `smallest_key`. Read next entry.
                {
                    for (idx, f) in idx_to_file.iter_mut() {
                        let f_smallest_key = idx_to_last_key.get(idx).unwrap();
                        if f_smallest_key.is_none() {
                            // No keys left.
                            continue;
                        }
                        if f_smallest_key.as_ref().unwrap() != smallest_key.as_ref().unwrap() {
                            // Has a different smallest key.
                            continue;
                        }
                        println!(
                            "discarding key {} for file {:04}.dat",
                            f_smallest_key.as_ref().unwrap(),
                            idx
                        );
                        // Read next entry.
                        let _ = self.read_value(f)?; // Ignore value.
                        let next_key = self.read_key(f)?;
                        idx_to_last_key.insert(*idx, next_key);
                    }
                }
            }
        } // outfile

        // Remove all data files.
        for file_idx in 0..count {
            let filepath = datapath.join(format!("{:04}.dat", file_idx));
            std::fs::remove_file(filepath)?;
        }

        // Rename merged.dat to 0000.dat.
        let from = datapath.join("merged.dat");
        let to = datapath.join("0000.dat");
        std::fs::rename(from, to)?;

        // Update count.txt to 1.
        self.write_count(1)?;

        println!("merge ... end");

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
    assert!(got.is_ok());
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
        assert!(got.is_ok());
        // Expect an error on the subsequent insert.
        let got = sst.insert_str("b".to_string(), "c".to_string());
        assert!(got.is_err());
    }

    // Test overwriting existing values.
    {
        // Insert a key with 1 byte, and a value with MAX_SIZE - 1 bytes.
        let got = sst.insert_str("a".to_string(), large_str);
        assert!(got.is_ok());
        // Expect no error on overwrite.
        let got = sst.insert_str("a".to_string(), "b".to_string());
        assert!(got.is_ok());
    }
}

struct TempFile {
    path: std::path::PathBuf,
}

impl TempFile {
    fn new(path: &std::path::Path) -> Self {
        // Ignore error in case file does not exist. Test may have exited before creating the file.
        std::fs::remove_file(path);
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
        )
        .expect("should insert");
        lsm1.insert_str("shared_key".to_string(), "value_from_thread1".to_string())
            .expect("should insert");
    });

    let mut lsm2 = lsm.clone();
    let handle2 = thread::spawn(move || {
        lsm2.insert_str(
            "key_from_thread2".to_string(),
            "value_from_thread2".to_string(),
        )
        .expect("should insert");
        lsm2.insert_str("shared_key".to_string(), "value_from_thread2".to_string())
            .expect("should insert");
    });
    handle1.join().expect("handle1 should join");
    handle2.join().expect("handle2 should join");

    let got = lsm
        .find_str("key_from_thread1".to_string())
        .expect("should have value for 'key_from_thread1'")
        .unwrap();
    assert_eq!(got, "value_from_thread1".to_string());

    let got = lsm
        .find_str("key_from_thread2".to_string())
        .expect("should have value for 'key_from_thread2'")
        .unwrap();
    assert_eq!(got, "value_from_thread2".to_string());

    let got = lsm
        .find_str("shared_key".to_string())
        .expect("should have value for `shared_key`")
        .unwrap();
    assert!(got == "value_from_thread1" || got == "value_from_thread2");
}

struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new(path: &std::path::Path) -> Self {
        // Ignore error in case directory does not exist. Test may have exited before creating the file.
        std::fs::remove_dir_all(path).expect("should remove all");
        std::fs::create_dir(path.clone()).expect("can create directory");
        return TempDir {
            path: path.to_path_buf(),
        };
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Ignore error in case directory does not exist. Test may have exited before creating the file.
        // std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn LSM_can_insert_and_write_to_disk() {
    let datadir = TempDir::new(&PathBuf::from("./data"));
    let largestr = String::from("a").repeat(SSTableInMemory::MAX_SIZE - 1);
    let mut lsm = LSM::new();
    {
        let res = lsm.insert_str("a".to_string(), largestr.clone());
        assert!(res.is_ok());
    }
    // Insert again. Expect existing SSTable to be written to disk.
    {
        let datafile0 = datadir.path.join("0000.dat");
        assert!(!datafile0.exists());
        let res = lsm.insert_str("b".to_string(), largestr.clone());
        assert!(res.is_ok(), "err={:?}", res.err().unwrap());
        assert!(datafile0.exists());
    }

    // Expect "b" to have been inserted.
    {
        let got = lsm
            .find_str("b".to_string())
            .expect("should find")
            .expect("should have a value");
        assert_eq!(got, largestr);
    }

    // Expect "a" (written to disk) can be found.
    {
        let got = lsm
            .find_str("a".to_string())
            .expect("should find")
            .expect("should have a value");
        assert_eq!(got, largestr);
    }

    // Insert again. Expect existing SSTable to be written to disk.
    {
        let datafile1 = datadir.path.join("0001.dat");
        assert!(!datafile1.exists());
        let res = lsm.insert_str("c".to_string(), largestr.clone());
        assert!(res.is_ok(), "err={:?}", res.err().unwrap());
        assert!(datafile1.exists());
    }

    // Expect "a" (written to disk) can be found.
    {
        let got = lsm
            .find_str("a".to_string())
            .expect("should find")
            .expect("should have a value");
        assert_eq!(got, largestr);
    }

    // Expect "b" (written to disk) can be found.
    {
        let got = lsm
            .find_str("b".to_string())
            .expect("should find")
            .expect("should have a value");
        assert_eq!(got, largestr);
    }
}

#[test]
fn LSM_can_merge() {
    let datadir = TempDir::new(&PathBuf::from("./data"));
    let largestr = String::from("a").repeat(SSTableInMemory::MAX_SIZE - 1);
    let mut lsm = LSM::new();
    {
        let res = lsm.insert_str("a".to_string(), largestr.clone());
        assert!(res.is_ok());
    }
    // Insert again. Expect existing SSTable to be written to disk.
    {
        let datafile0 = datadir.path.join("0000.dat");
        assert!(!datafile0.exists());
        let res = lsm.insert_str("b".to_string(), largestr.clone());
        assert!(res.is_ok(), "err={:?}", res.err().unwrap());
        assert!(datafile0.exists());
    }
    // Insert again. Expect existing SSTable to be written to disk.
    {
        let datafile1 = datadir.path.join("0001.dat");
        assert!(!datafile1.exists());
        let res = lsm.insert_str("c".to_string(), largestr.clone());
        assert!(res.is_ok(), "err={:?}", res.err().unwrap());
        assert!(datafile1.exists());
    }

    // Merge.
    {
        let datafile0 = datadir.path.join("0000.dat");
        let datafile1 = datadir.path.join("0001.dat");
        assert!(datafile0.exists());
        assert!(datafile1.exists());
        assert_eq!(lsm.read_count().expect("should read count"), 2);
    }
    lsm.merge().expect("should merge");
    {
        let datafile0 = datadir.path.join("0000.dat");
        let datafile1 = datadir.path.join("0001.dat");
        assert!(datafile0.exists());
        assert!(!datafile1.exists());
        assert_eq!(lsm.read_count().expect("should read count"), 1);
    }

    // Expect "a" and "b" can both be found.
    // Expect "a" (written to disk) can be found.
    {
        let got = lsm
            .find_str("a".to_string())
            .expect("should find")
            .expect("should have a value");
        assert_eq!(got, largestr);
    }

    // Expect "b" (written to disk) can be found.
    {
        let got = lsm
            .find_str("b".to_string())
            .expect("should find")
            .expect("should have a value");
        assert_eq!(got, largestr);
    }
}

// #[test]
// fn LSM_can_merge_and_overwrite() {
//     let datadir = TempDir::new(&PathBuf::from("./data"));
//     let largestr1 = String::from("a").repeat(SSTableInMemory::MAX_SIZE - 1);
//     let largestr2 = String::from("b").repeat(SSTableInMemory::MAX_SIZE - 1);
//     let mut lsm = LSM::new();
//     {
//         let res = lsm.insert_str("a".to_string(), largestr1.clone());
//         assert!(res.is_ok());
//     }
//     // Insert again. Expect existing SSTable to be written to disk.
//     {
//         let datafile0 = datadir.path.join("0000.dat");
//         assert!(!datafile0.exists());
//         let res = lsm.insert_str("b".to_string(), largestr1.clone());
//         assert!(res.is_ok(), "err={:?}", res.err().unwrap());
//         assert!(datafile0.exists());
//     }
//     // Insert "a" again with a different value. Expect existing SSTable to be written to disk.
//     {
//         let datafile1 = datadir.path.join("0001.dat");
//         assert!(!datafile1.exists());
//         let res = lsm.insert_str("a".to_string(), largestr2.clone());
//         assert!(res.is_ok(), "err={:?}", res.err().unwrap());
//         assert!(datafile1.exists());
//     }
//     // Insert again. Expect existing SSTable to be written to disk.
//     {
//         let datafile0 = datadir.path.join("0002.dat");
//         assert!(!datafile0.exists());
//         let res = lsm.insert_str("c".to_string(), largestr1.clone());
//         assert!(res.is_ok(), "err={:?}", res.err().unwrap());
//         assert!(datafile0.exists());
//     }

//     // Expect 0000.dat to contain "a" with `largestr1`. 0002.dat to contain "a" with `largestr2`.

//     // Merge.
//     {
//         let datafile0 = datadir.path.join("0000.dat");
//         let datafile1 = datadir.path.join("0001.dat");
//         let datafile2 = datadir.path.join("0002.dat");
//         assert!(datafile0.exists());
//         assert!(datafile1.exists());
//         assert!(datafile2.exists());
//         assert_eq!(lsm.read_count().expect("should read count"), 3);
//     }
//     lsm.merge().expect("should merge");
//     {
//         let datafile0 = datadir.path.join("0000.dat");
//         let datafile1 = datadir.path.join("0001.dat");
//         let datafile2 = datadir.path.join("0002.dat");
//         assert!(datafile0.exists());
//         assert!(!datafile1.exists());
//         assert!(!datafile2.exists());
//         assert_eq!(lsm.read_count().expect("should read count"), 1);
//     }

//     // Expect "a" (written to disk) can be found with `largestr2`.
//     {
//         let got = lsm
//             .find_str("a".to_string())
//             .expect("should find")
//             .expect("should have a value");
//         assert_eq!(got, largestr2);
//     }
// }
