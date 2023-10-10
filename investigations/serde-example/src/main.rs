// Bson - enum.
// Document - random access.
// RawDocumentBuf - owned. Backed by bytes.

use mongodb::bson;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    pages: i32,
}

fn main() {
    let book = Book {
        title: "foo".to_string(),
        pages: 123,
    };

    // struct => BSON data.
    {
        let book_bson = bson::to_bson(&book).unwrap();
        let mut book_bytes = Vec::<u8>::new();
        book_bson
            .as_document()
            .unwrap()
            .to_writer(&mut book_bytes)
            .expect("should write bytes");
        println!("struct => BSON data: {:?}", book_bytes);
    }

    // BSON data => struct.
    {
        let book_bytes: [u8; 31] = [
            31, 0, 0, 0, 2, 116, 105, 116, 108, 101, 0, 4, 0, 0, 0, 102, 111, 111, 0, 16, 112, 97,
            103, 101, 115, 0, 123, 0, 0, 0, 0,
        ];
        let book_document = bson::Document::from_reader(&mut book_bytes.as_slice()).unwrap();
        let book_bson: bson::Bson = bson::Bson::Document(book_document);
        let book_struct: Book = bson::from_bson(book_bson).unwrap();
        println!("BSON data => struct: {:?}", book_struct);
    }

    // struct => EJSON.
    {
        let book_bson = bson::to_bson(&book).unwrap();
        let book_ejson: serde_json::Value = book_bson.into_canonical_extjson();
        let book_ejson_str = book_ejson.to_string();
        println!("struct => EJSON: {:?}", book_ejson_str);
    }

    // EJSON => struct.
    {
        let book_ejson_str = "{\"title\": \"foo\", \"pages\": { \"$numberInt\": \"123\" } }";
        let book_ejson: serde_json::Value = serde_json::from_str(book_ejson_str).unwrap();
        let book_bson: bson::Bson = book_ejson.try_into().unwrap();
        let book_struct: Book = bson::from_bson(book_bson).unwrap();
        println!("EJSON => struct: {:?}", book_struct);
    }

    // BSON data => EJSON.
    {
        let book_bytes: [u8; 31] = [
            31, 0, 0, 0, 2, 116, 105, 116, 108, 101, 0, 4, 0, 0, 0, 102, 111, 111, 0, 16, 112, 97,
            103, 101, 115, 0, 123, 0, 0, 0, 0,
        ];
        let book_document = bson::Document::from_reader(&mut book_bytes.as_slice()).unwrap();
        let book_bson: bson::Bson = bson::Bson::Document(book_document);
        let book_ejson: serde_json::Value = book_bson.into_canonical_extjson();
        let book_ejson_str = book_ejson.to_string();
        println!("BSON data => EJSON: {:?}", book_ejson_str);
    }

    // EJSON => BSON data.
    {
        let book_ejson_str = "{\"title\": \"foo\", \"author\": \"bar\"}";
        let book_ejson: serde_json::Value = serde_json::from_str(book_ejson_str).unwrap();
        let book_bson: bson::Bson = book_ejson.try_into().unwrap();
        let mut book_bytes = Vec::<u8>::new();
        book_bson
            .as_document()
            .unwrap()
            .to_writer(&mut book_bytes)
            .expect("should write bytes");
        println!("EJSON => BSON data: {:?}", book_bytes);
    }
}
