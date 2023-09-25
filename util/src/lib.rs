use mongodb::error::Result;
use mongodb::event::command::{
    CommandEventHandler, CommandFailedEvent, CommandStartedEvent, CommandSucceededEvent,
};
use mongodb::options::ClientOptions;
use std::sync::Arc;

// Sample usage:
// let opts = util::make_opts_with_monitor("mongodb://localhost:27017").await?;
pub struct CommandHandlerDumper {
    command_started_only: bool,
}

impl CommandEventHandler for CommandHandlerDumper {
    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command is initiated.
    fn handle_command_started_event(&self, event: CommandStartedEvent) {
        println!("CommandStartedEvent: {:?}", event)
    }

    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command successfully completes.
    fn handle_command_succeeded_event(&self, event: CommandSucceededEvent) {
        if self.command_started_only {
            return;
        }
        println!("CommandSucceededEvent: {:?}", event)
    }

    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command fails to complete successfully.
    fn handle_command_failed_event(&self, event: CommandFailedEvent) {
        if self.command_started_only {
            return;
        }
        println!("CommandFailedEvent: {:?}", event)
    }
}

pub async fn make_opts_with_monitor(uri_str: &str) -> Result<ClientOptions> {
    let mut opts = ClientOptions::parse(uri_str).await?;
    opts.command_event_handler = Some(Arc::new(CommandHandlerDumper {
        command_started_only: true,
    }));
    return Ok(opts);
}
