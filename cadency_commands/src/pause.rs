use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::model::colour::Colour;
use serenity::{async_trait, client::Context, model::application::CommandInteraction};

#[derive(CommandBaseline, Default)]
#[description = "Pause the current song"]
#[deferred = true]
pub struct Pause {}

#[async_trait]
impl CadencyCommand for Pause {
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
                .title("⏸️ Pause")
                .color(Colour::from_rgb(255, 165, 0)) // Orange
                .description("❌ **Nothing to pause**\n\nThere are no tracks currently playing.");
            Ok(response_builder.embeds(vec![embed]).build()?)
        } else {
            handler.queue().pause().map_err(|err| {
                error!("Failed to pause: {err:?}");
                CadencyError::Command {
                    message: "❌ **Could not pause the track**".to_string(),
                }
            })?;
            let embed = serenity::builder::CreateEmbed::default()
                .title("⏸️ Pause")
                .color(Colour::from_rgb(255, 165, 0)) // Orange
                .description(
                    "✅ **Paused**\n\nPlayback has been paused. Use `/resume` to continue.",
                );
            Ok(response_builder.embeds(vec![embed]).build()?)
        }
    }
}
