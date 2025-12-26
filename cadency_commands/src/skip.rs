use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use serenity::model::colour::Colour;

#[derive(CommandBaseline, Default)]
#[description = "Skip current song"]
#[deferred = true]
pub struct Skip {}

#[async_trait]
impl CadencyCommand for Skip {
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
                .title("⏭️ Skip")
                .color(Colour::from_rgb(255, 215, 0)) // Gold
                .description("❌ **Nothing to skip**\n\nThere are no tracks currently playing.");
            Ok(response_builder.embeds(vec![embed]).build()?)
        } else {
            handler.queue().skip().map_err(|err| {
                error!("Failed to skip: {err:?}");
                CadencyError::Command {
                    message: "❌ **Could not skip the track**".to_string(),
                }
            })?;
            let embed = serenity::builder::CreateEmbed::default()
                .title("⏭️ Skip")
                .color(Colour::from_rgb(255, 215, 0)) // Gold
                .description("✅ **Skipped**\n\nMoving to the next track in queue.");
            Ok(response_builder.embeds(vec![embed]).build()?)
        }
    }
}