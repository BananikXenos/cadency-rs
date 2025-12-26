use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use serenity::model::colour::Colour;

#[derive(CommandBaseline, Default)]
#[description = "Resume current song if paused"]
#[deferred = true]
pub struct Resume {}

#[async_trait]
impl CadencyCommand for Resume {
    async fn execute<'a>(
        &self,
        ctx: &Context,
        command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let guild_id = command.guild_id.ok_or(CadencyError::Command {
            message: "❌ **This command can only be executed on a server**".to_string(),
        })?;
        let manager = utils::voice::get_songbird(ctx).await;
        let call = manager.get(guild_id).ok_or(CadencyError::Command {
            message: "❌ **No active voice session on the server**".to_string(),
        })?;
        let handler = call.lock().await;
        if handler.queue().is_empty() {
            let embed = serenity::builder::CreateEmbed::default()
                .title("▶️ Resume")
                .color(Colour::from_rgb(0, 255, 0)) // Lime
                .description("❌ **Nothing to resume**\n\nThere are no tracks in the queue.");
            Ok(response_builder.embeds(vec![embed]).build()?)
        } else {
            handler.queue().resume().map_err(|err| {
                error!("Failed to resume: {err:?}");
                CadencyError::Command {
                    message: "❌ **Could not resume**".to_string(),
                }
            })?;
            let embed = serenity::builder::CreateEmbed::default()
                .title("▶️ Resume")
                .color(Colour::from_rgb(0, 255, 0)) // Lime
                .description("✅ **Resumed**\n\nPlayback has been resumed!");
            Ok(response_builder.embeds(vec![embed]).build()?)
        }
    }
}