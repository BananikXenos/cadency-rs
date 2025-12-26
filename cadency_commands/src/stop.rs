use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use serenity::model::colour::Colour;

#[derive(CommandBaseline, Default)]
#[description = "Stop music and clear the track list"]
#[deferred = true]
pub struct Stop {}

#[async_trait]
impl CadencyCommand for Stop {
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

        let embed = if handler.queue().is_empty() {
            serenity::builder::CreateEmbed::default()
                .title("🛑 Stop & Clear")
                .color(Colour::from_rgb(255, 69, 0)) // Orange red
                .description("❌ **Nothing to stop**\n\nThere are no tracks in the queue.")
        } else {
            handler.queue().stop();
            serenity::builder::CreateEmbed::default()
                .title("🛑 Stop & Clear")
                .color(Colour::from_rgb(255, 69, 0)) // Orange red
                .description("✅ **Stopped**\n\nCleared the queue and stopped playback.")
        };

        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}