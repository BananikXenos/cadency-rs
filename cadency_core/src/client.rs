use crate::{
    command::{Commands, CommandsScope},
    error::CadencyError,
    handler::command::Handler,
    http::HttpClientKey,
    intents::CadencyIntents,
    CadencyCommand,
};
use ctrlc;
use log::info;
use serenity::{client::Client, model::gateway::GatewayIntents};
use songbird::SerenityInit;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::oneshot;

#[derive(derive_builder::Builder)]
pub struct Cadency {
    token: String,
    #[builder(default)]
    commands: Vec<Arc<dyn CadencyCommand>>,
    #[builder(default = "CadencyIntents::default().into()")]
    intents: GatewayIntents,
    /// Used when registering commands with the Discord API
    #[builder(default)]
    commands_scope: CommandsScope,
}

impl Cadency {
    #[must_use]
    pub fn builder() -> CadencyBuilder {
        CadencyBuilder::default()
    }

    /// This will actually start the configured Cadency bot
    pub async fn start(self) -> Result<(), CadencyError> {
        // Build the serenity client as before
        let mut client = Client::builder(self.token, self.intents)
            .event_handler(Handler)
            .register_songbird()
            .type_map_insert::<Commands>(self.commands)
            .type_map_insert::<HttpClientKey>(reqwest::Client::new())
            .type_map_insert::<CommandsScope>(self.commands_scope)
            .await
            .map_err(|err| CadencyError::Start {
                source: Box::new(err),
            })?;

        // Keep a handle to the shard manager so we can shut down gracefully on Ctrl+C
        let shard_manager = client.shard_manager.clone();

        // Run the client's start future in a background task so we can await a signal concurrently
        let client_task = tokio::spawn(async move {
            client.start().await.map_err(|err| CadencyError::Start {
                source: Box::new(err),
            })
        });

        // Use ctrlc crate to listen for SIGINT (cross-platform). We'll notify via a oneshot.
        let (tx, rx) = oneshot::channel();
        // ctrlc handlers must be 'static; wrap sender in Arc<Mutex<..>> so closure can take ownership.
        let tx = Arc::new(StdMutex::new(Some(tx)));
        ctrlc::set_handler({
            let tx = Arc::clone(&tx);
            move || {
                if let Ok(mut guard) = tx.lock() {
                    if let Some(tx) = guard.take() {
                        let _ = tx.send(());
                    }
                }
            }
        })
        .map_err(|err| CadencyError::Runtime(format!("Failed to set ctrlc handler: {}", err)))?;

        // Wait for the signal
        rx.await
            .map_err(|_| CadencyError::Runtime("Failed to receive ctrlc signal".to_string()))?;

        info!("Received Ctrl+C - shutting down shards");

        // Ask the shard manager to shut down all shards gracefully
        shard_manager.shutdown_all().await;

        // Wait for the client task to finish and forward any error
        match client_task.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(join_err) => Err(CadencyError::Runtime(format!(
                "Client task join error: {}",
                join_err
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fail_to_build_without_token() {
        let build = Cadency::builder().commands(vec![]).build();
        assert!(build.is_err())
    }

    #[test]
    fn build_with_token() {
        let build = Cadency::builder()
            .commands(vec![])
            .token("some-token".to_string())
            .build();
        assert!(build.is_ok())
    }

    #[test]
    fn build_with_default_intents() {
        let build = Cadency::builder()
            .commands(vec![])
            .token("some-token".to_string())
            .build()
            .unwrap();
        assert_eq!(build.intents, CadencyIntents::default().into());
    }

    #[test]
    fn build_with_empty_commands() {
        let build = Cadency::builder()
            .commands(vec![])
            .token("some-token".to_string())
            .build()
            .unwrap();
        assert!(build.commands.is_empty());
    }
}
