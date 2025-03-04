// Bson - enum.
// Document - random access.
// RawDocumentBuf - owned. Backed by bytes.

use bson::{self, Bson};
use serde::de::{MapAccess, Visitor};
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::fmt;

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
        println!("{:20}: {:?}", "struct => BSON data", book_bytes);
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
        println!("{:20}: {:?}", "BSON data => struct", book_struct);
    }

    // struct => EJSON.
    {
        let book_bson = bson::to_bson(&book).unwrap();
        let book_ejson: serde_json::Value = book_bson.into_canonical_extjson();
        let book_ejson_str = book_ejson.to_string();
        println!("{:20}: {}", "struct => EJSON", book_ejson_str);
    }

    // EJSON => struct.
    {
        let book_ejson_str = "{\"title\": \"foo\", \"pages\": { \"$numberInt\": \"123\" } }";
        let book_ejson: serde_json::Value = serde_json::from_str(book_ejson_str).unwrap();
        let book_bson: bson::Bson = book_ejson.try_into().unwrap();
        let book_struct: Book = bson::from_bson(book_bson).unwrap();
        println!("{:20}: {:?}", "EJSON => struct", book_struct);
    }

    // bson::Document to EJSON.
    {
        let mut doc = bson::Document::new();
        doc.insert("foo", 123);
        let ejson_str = Bson::Document(doc).into_canonical_extjson().to_string();
        println!("{:20}: {:?}", "Document => EJSON", ejson_str); // {"foo":{"$numberInt":"123"}}
    }

    // bson::Document => JSON.
    {
        let mut doc = bson::Document::new();
        doc.insert("foo", 123);
        let bson = Bson::Document(doc);
        let json_str = serde_json::to_string(&bson).expect("should serialize");
        println!("{:20}: {}", "Document => JSON", json_str); // {"foo":123}
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
        println!("{:20}: {:?}", "BSON data => EJSON", book_ejson_str);
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
        println!("{:20}: {:?}", "EJSON => BSON data", book_bytes);
    }

    // Implement Serialize and Deserialize.
    {
        // TimesTwo serializes `x` to twice the value.
        struct TimesTwo {
            x: i32,
        }
        impl Serialize for TimesTwo {
            // Required method
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                // "the Serialize implementation for the data structure is responsible for mapping the data structure into the Serde data model by invoking exactly one of the Serializer methods"
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("x")?;
                let val = self.x * 2;
                map.serialize_value(&val)?;
                return map.end();
            }
        }

        struct TimesTwoVisitor;

        impl<'de> Visitor<'de> for TimesTwoVisitor {
            type Value = TimesTwo;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map mapping 'x' to an i32")
            }

            fn visit_map<A>(self, mut value: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let _: String = value.next_key()?.unwrap();
                let value: i32 = value.next_value().unwrap();
                return Ok(TimesTwo { x: value / 2 });
            }
        }

        impl<'de> Deserialize<'de> for TimesTwo {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_map(TimesTwoVisitor)
            }
        }

        let tt = TimesTwo { x: 123 };
        let serialized = serde_json::to_string(&tt).expect("should serialize");
        assert_eq!(serialized, r#"{"x":246}"#);
        // Q: Why does this result in "trailing characters" error?
        // A: I think because the visitor was not consuming the map input.
        let deserialized: TimesTwo = serde_json::from_str(&serialized).expect("should deserialize");
        assert_eq!(deserialized.x, tt.x);
    }
}
