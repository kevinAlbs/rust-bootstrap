use bson::Document;

fn main() {
    // `bytes` is BSON for `{"foo" : {"$numberInt": "123"}}`.
    // The `$numberInt` is a key, not EJSON!
    let hex = "2300000003666f6f001900000002246e756d626572496e740004000000313233000000";
    let bytes = hex::decode(hex).unwrap();
    bytes.as_mut_slice()
    let doc = Document::from_reader(&mut bytes.as_slice()).unwrap();
    // The '$numberInto' is misinterpreted as EJSON!
    println!("{}", doc); // { "foo": 123 }
    
    // Compared with pymongo:
    // import bson
    // bson.decode(bytes.fromhex("2300000003666f6f001900000002246e756d626572496e740004000000313233000000"))
    // {'foo': {'$numberInt': '123'}}
}
