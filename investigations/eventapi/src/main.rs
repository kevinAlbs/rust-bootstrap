use mongodb::bson::doc;
use mongodb::error::Result;
use mongodb::event::command::{CommandEventHandler, CommandStartedEvent};
use mongodb::options::ClientOptions;
use mongodb::Client;
use std::sync::Arc;
use tokio;

struct SimpleEventHandler;
impl CommandEventHandler for SimpleEventHandler {
    fn handle_command_started_event(&self, event: CommandStartedEvent) {
        println!("command started: {}", event.command);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let handler: Arc<dyn CommandEventHandler> = Arc::new(SimpleEventHandler {});
    let opts = ClientOptions::builder()
        .command_event_handler(handler)
        .build();
    let client = Client::with_options(opts)?;

    client
        .database("db")
        .run_command(doc! {"ping": 1}, None)
        .await?;

    return Ok(());
}
