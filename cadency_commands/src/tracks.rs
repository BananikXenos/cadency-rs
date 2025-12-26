use cadency_core::{
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::Context,
    model::application::CommandInteraction,
    model::colour::Colour,
};
use songbird::{input::AuxMetadata, tracks::LoopState};

#[derive(CommandBaseline, Default)]
#[description = "List all tracks in the queue"]
#[deferred = true]
pub struct Tracks {}

#[async_trait]
impl CadencyCommand for Tracks {
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
        let response_builder = if handler.queue().is_empty() {
            response_builder.message(Some("❌ **No tracks in the queue**\n\nUse `/play` to add some music!".to_string()))
        } else {
            let queue_snapshot = handler.queue().current_queue();
            let mut embeded_tracks = CreateEmbed::default()
                .color(Colour::from_rgb(114, 137, 218)) // Discord blurple
                .title("🎵 Track Queue")
                .description(format!("📊 **Total Tracks:** {}", queue_snapshot.len()));

            for (index, track) in queue_snapshot.into_iter().enumerate() {
                let track_position = index + 1;
                let (title, url, loop_state) = {
                    let metadata = track.data::<AuxMetadata>();
                    let title = metadata
                        .title
                        .as_ref()
                        .map_or("Unknown Title", |t| t);
                    let url = metadata
                        .source_url
                        .as_ref()
                        .map_or("No URL", |u| u);
                    let track_info = track.get_info().await.unwrap();
                    (title.to_owned(), url.to_owned(), track_info.loops)
                };

                let mut embed_value = if url != "No URL" {
                    format!("🔗 [View Source]({})", url)
                } else {
                    "🔗 No URL available".to_string()
                };

                match loop_state {
                    LoopState::Infinite => {
                        embed_value.push_str("\n🔁 **Loop:** Infinite");
                    }
                    LoopState::Finite(loop_amount) => {
                        if loop_amount > 0 {
                            embed_value.push_str(&format!("\n🔁 **Loop:** {} times", loop_amount));
                        }
                    }
                }

                let field_name = if index == 0 {
                    format!("▶️ {}. {}", track_position, title)
                } else {
                    format!("{}. {}", track_position, title)
                };

                embeded_tracks = embeded_tracks.field(field_name, embed_value, false);
            }
            response_builder.embeds(vec![embeded_tracks])
        };
        Ok(response_builder.build()?)
    }
}