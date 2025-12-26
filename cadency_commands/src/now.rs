use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use serenity::model::colour::Colour;
use songbird::{input::AuxMetadata, tracks::LoopState};

#[derive(CommandBaseline, Default)]
#[description = "Shows current song"]
pub struct Now {}

#[async_trait]
impl CadencyCommand for Now {
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

        let metadata = track.data::<AuxMetadata>();
        let loop_state = track.get_info().await.unwrap().loops;

        let default_title = "Unknown Song".to_string();
        let title = metadata.title.as_ref().unwrap_or(&default_title);
        let url = metadata.source_url.as_ref();

        let mut description = format!("🎵 **Now Playing:** `{}`", title);

        if let Some(url) = url {
            description.push_str(&format!("\n🔗 **Link:** [View Source]({})", url));
        }

        match loop_state {
            LoopState::Infinite => {
                description.push_str("\n🔁 **Loop:** Infinite");
            }
            LoopState::Finite(count) if count > 0 => {
                description.push_str(&format!("\n🔁 **Loop:** {} times remaining", count));
            }
            _ => {}
        }

        let embed = serenity::builder::CreateEmbed::default()
            .title("🎧 Now Playing")
            .color(Colour::from_rgb(255, 110, 64)) // Coral
            .description(description);
        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}