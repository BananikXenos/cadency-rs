use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::model::colour::Colour;
use serenity::{async_trait, client::Context, model::application::CommandInteraction};

#[derive(Default, CommandBaseline)]
#[name = "loop"]
#[description = "Loop the current track"]
#[argument(
    name = "amount",
    description = "The amount of times to loop the track",
    required = false,
    kind = "Integer"
)]
#[allow(clippy::duplicated_attributes)]
#[argument(
    name = "stop",
    description = "Cancel looping",
    required = false,
    kind = "Boolean"
)]
pub struct TrackLoop {}

#[async_trait]
impl CadencyCommand for TrackLoop {
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
        let track = handler.queue().current().ok_or(CadencyError::Command {
            message: "❌ **No song is playing**".to_string(),
        })?;

        let loop_amount = self.arg_amount(command);
        let stop_argument = self.arg_stop(command);

        let mut embed = serenity::builder::CreateEmbed::default()
            .title("🔁 Track Loop")
            .color(Colour::from_rgb(255, 140, 0)); // Dark orange

        if let Some(stop) = stop_argument {
            if stop {
                track.disable_loop().map_err(|err| {
                    error!("Could not disable loop: {}", err);
                    CadencyError::Command {
                        message: "❌ **Could not disable loop**".to_string(),
                    }
                })?;
                embed = embed.description("✅ **Loop Disabled**\n\nThe track will play only once.");
                return Ok(response_builder.embeds(vec![embed]).build()?);
            }
        }

        if let Some(amount) = loop_amount {
            track.loop_for(amount as usize).map_err(|err| {
                error!("Could not loop track '{amount}' times: {}", err);
                CadencyError::Command {
                    message: "❌ **Could not loop track**".to_string(),
                }
            })?;
            embed = embed.description(format!(
                "✅ **Loop Enabled**\n\n🔁 The current track will loop **{}** times.",
                amount
            ));
        } else {
            track.enable_loop().map_err(|err| {
                error!("Could not loop track infinite: {}", err);
                CadencyError::Command {
                    message: "❌ **Could not loop track**".to_string(),
                }
            })?;
            embed = embed.description(
                "✅ **Loop Enabled**\n\n🔁 The current track will loop **infinitely**.",
            );
        }
        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}
