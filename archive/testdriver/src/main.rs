use mongodb::error::Result;
use mongodb::event::command::{
    CommandEventHandler, CommandFailedEvent, CommandStartedEvent, CommandSucceededEvent,
};
use mongodb::options::ClientOptions;
use mongodb::sync::Client;
use std::sync::Arc;

use mongodb::bson;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::sync::Collection;

use serde::Serialize;

// This trait is required to use `try_next()` on the cursor
use futures::stream::TryStreamExt;

struct CommandHandlerDumper {}

impl CommandEventHandler for CommandHandlerDumper {
    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command is initiated.
    fn handle_command_started_event(&self, event: CommandStartedEvent) {
        println!("CommandStartedEvent: {:?}", event)
    }

    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command successfully completes.
    fn handle_command_succeeded_event(&self, event: CommandSucceededEvent) {
        println!("CommandSucceededEvent: {:?}", event)
    }

    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command fails to complete successfully.
    fn handle_command_failed_event(&self, event: CommandFailedEvent) {
        println!("CommandFailedEvent: {:?}", event)
    }
}

#[derive(Serialize)]
struct Foo {
    field: i32,
}

fn main() -> Result<()> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017")?;

    client_options.command_event_handler = Option::Some(Arc::new(CommandHandlerDumper {}));

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None)? {
        println!("{}", db_name);
    }

    Ok(())
}
