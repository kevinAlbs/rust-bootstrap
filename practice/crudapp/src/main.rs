// DONE: get `exit` test passing.
// DONE: get remaining tests passing
// DONE: implement loop.
// DONE: have `get_client` return a Result instead of panic.
// Q: How do I return an error? A: One option: use a result with a boxed `dyn Error`:
// std::result::Result<T, Box<dyn std::error::Error>>

use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, ServerApi, ServerApiVersion},
    sync::Client,
};
use std::{io, str::FromStr};

// `process_cmd` returns false if the command signals exit.
fn process_cmd(db: &mongodb::sync::Database, cmd: &str) -> Result<bool, String> {
    let coll = db.collection("entry");
    if cmd.starts_with("exit") {
        return Ok(false);
    }
    if cmd.starts_with("add ") {
        let remaining = &cmd["add ".len()..].trim();
        let mut d = Document::new();
        d.insert("contents", remaining);
        let res = coll.insert_one(d, None);
        if res.is_err() {
            return Err(String::from("error on insert"));
        }
        return Ok(true);
    }
    if cmd.starts_with("list") {
        let cur = coll.find(None, None).expect("should find");
        for doc in cur {
            let doc = doc.expect("should be document");
            let id = doc.get_object_id("_id").expect("should have ObjectID _id");
            let contents = doc
                .get_str("contents")
                .expect("should have string contents");
            println!("id: {}", id.to_hex());
            println!("contents:\n{}", contents);
            println!("");
        }
    }
    if cmd.starts_with("delete ") {
        let remaining = cmd["delete ".len()..].trim();
        let oid =
            mongodb::bson::oid::ObjectId::from_str(remaining).expect("should parse as ObjectID");
        println!("attempting to delete entry with OID: {}", oid.to_string());
        let mut query = Document::new();
        query.insert("_id", oid);
        coll.delete_one(query, None).expect("should delete");
    }
    return Ok(true);
}

fn get_client() -> mongodb::error::Result<mongodb::sync::Client> {
    // Replace the placeholder with your Atlas connection string
    let uri = "mongodb://localhost:27021";
    let mut client_options = ClientOptions::parse(uri)?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;

    // Send a ping to confirm a successful connection
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)?;

    return Ok(client);
}

#[test]
fn test_process_cmd() {
    let client = get_client().expect("should return client");
    let db = client.database("test");
    let coll: mongodb::sync::Collection<Document> = db.collection("entry");
    coll.drop(None).expect("should drop");

    // Check that `exit` causes a false return.
    {
        let got = process_cmd(&db, "exit");
        assert!(got.is_ok());
        assert!(got.unwrap() == false);
    }

    let id;

    // Check that 'add <text>' adds an entry to the test database.
    {
        let got = process_cmd(&db, "add foobar");
        assert!(got.is_ok());
        assert!(got.unwrap() == true);
        let cur = coll.find(None, None).expect("should find");
        let got = cur.deserialize_current().expect("should deserialize");
        id = got.get_object_id("_id").expect("should have _id").clone();
        let got = got.get_str("contents").expect("should have 'contents'");
        assert_eq!(got, "foobar");
    }

    // Check that 'delete' results in deleting.
    {
        let cmd = format!("delete {}", id.to_string());
        let got = process_cmd(&db, &cmd);
        assert!(got.is_ok());
        assert!(got.unwrap() == true);
        let cur = coll.find(None, None).expect("should find");
        assert_eq!(cur.count(), 0);
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Read CLI input to read an entry.
    println!("Journal commands:");
    println!("add <text>");
    println!("list");
    println!("delete <id>");
    println!("exit");

    let client = get_client()?;
    client.database("journal").drop(None).expect("should drop");

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("should read line");

        let tokens: Vec<&str> = input.split_ascii_whitespace().collect();
        if tokens.len() == 0 {
            println!("expected non-empty input. Exitting");
            return Ok(());
        }

        let got = process_cmd(&client.database("journal"), &input);
        match got {
            Ok(ret) => {
                if ret {
                    continue;
                } else {
                    println!("Goodbye");
                    return Ok(());
                }
            }
            Err(s) => {
                println!("Error processing command: {}", s);
                return Ok(());
            }
        }
    }
}
