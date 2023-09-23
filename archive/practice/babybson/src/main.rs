struct BSONBuilder {
    data : Vec<u8>,
    // index_stack is a stack of indexes pointing to the first byte of a length in a nested document.
    index_stack : Vec<usize>,
}

impl BSONBuilder {
    fn new () -> BSONBuilder {
        return BSONBuilder{
            // Add space for the length.
            data: vec![0, 0, 0, 0],
            index_stack: Vec::new(),
        };
    }

    fn usize_to_i32 (x : usize) -> i32 {
        if x > i32::MAX.try_into().unwrap() {
            panic!("value {} is too large. Cannot fit in i32", x);
        }
        return x.try_into().unwrap();
    }

    fn _append_key (&mut self, key : &str) {
        for b in key.bytes() {
            self.data.push(b);
        }
        // Push NULL byte.
        self.data.push(0);
    }

    fn _append_bytes (&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn append_int32 (&mut self, key : &str, value : i32) {
        // Append type.
        self.data.push(0x10);
        // Append key.
        self._append_key (&key);
        // Convert value to Little Endian and append.
        let bytes = value.to_le_bytes();
        self._append_bytes (&bytes);
    }

    fn start_document (&mut self, key : &str) {
        // Append type.
        self.data.push(0x03);
        // Append key.
        self._append_key (key);
        // Save position for location of length bytes.
        self.index_stack.push(self.data.len());
        // Reserve length.
        self.data.extend_from_slice(&[0;4]);
    }

    fn end_document (&mut self) {
        if self.index_stack.len() == 0 {
            panic!("Attempting to end document, but no document is being appended.");
        }
        let start_index = self.index_stack.pop().unwrap();
        // Push null byte.
        self.data.push(0x00);
        let document_len = BSONBuilder::usize_to_i32 (self.data.len() - start_index);
        let document_len_bytes = document_len.to_le_bytes();
        for i in 0..3 {
            self.data[start_index + i] = document_len_bytes[i];
        }
    }

    fn build(&mut self) -> Vec<u8> {
        // TODO: panic if there is an un-ended document.
        // Set the starting length of the document.
        // Add one for trailing NULL byte.
        let len = self.data.len() + 1;
        if len > i32::MAX.try_into().unwrap() {
            panic!("document too large");
        }
        let len : i32 = len.try_into().unwrap();
        let lenbytes = len.to_le_bytes();
        assert!(lenbytes.len() == 4);
        for i in 0..3 {
            self.data[i] = lenbytes[i];
        }

        // Add a NULL byte.
        self.data.push(0);

        // Swap and reset data.
        // `build` does not own `self.data`. Do a replace so `self.data` is not deinitialized.
        let toreturn = core::mem::replace(&mut self.data, vec![0, 0, 0, 0]);
        return toreturn;
    }
}

fn main() {
    println!("Hello, world!");
}


#[test]
fn test_empty () {
    let mut bb = BSONBuilder::new();
    let got = bb.build();
    assert_eq!(got, vec![5,0,0,0,0]);
}

#[test]
fn test_int32 () {
    let mut bb = BSONBuilder::new();
    bb.append_int32 ("x", 1);
    let got = bb.build();
    assert_eq!(got, vec![0x0c, 0x00, 0x00, 0x00, 0x10, 0x78, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn test_nested_document () {
    let mut bb = BSONBuilder::new();
    bb.start_document("x");
    bb.append_int32 ("y", 1);
    bb.end_document();
    let got = bb.build();
    assert_eq!(got, vec![0x14, 0x00, 0x00, 0x00, 0x03, 0x78, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x10, 0x79, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
}
