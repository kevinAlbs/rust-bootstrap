use bson::Document;

fn main() {
    // `bytes` represents the BSON document {"foo" : {"$numberInt": "123" }}. The `$numberInt` is not intended to be EJSON.
    let bytes = hex::decode("2300000003666f6f001900000002246e756d626572496e740004000000313233000000").unwrap();
    let doc = Document::from_reader(&mut bytes.as_slice()).unwrap();
    // The '$numberInto' is interpreted as EJSON into an int32!
    println!("{}", doc); // { "foo": 123 }
    
    // Compared with pymongo:
    // import bson
    // bson.decode(bytes.fromhex("2300000003666f6f001900000002246e756d626572496e740004000000313233000000"))
    // {'foo': {'$numberInt': '123'}}
}
