use cadency_core::{
    handler::voice::InactiveHandler,
    response::{Response, ResponseBuilder},
    utils, CadencyCommand, CadencyError,
};
use reqwest::Url;
use serenity::model::colour::Colour;
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use songbird::events::Event;

#[derive(CommandBaseline)]
#[description = "Play a song from Youtube"]
#[deferred = true]
#[argument(
    name = "query",
    description = "URL or search query like: 'Hey Jude Beatles'",
    kind = "String"
)]
pub struct Play {
    /// The maximum number of songs that can be added to the queue from a playlist
    playlist_song_limit: i32,
    /// The maximum length of a single song in seconds
    song_length_limit: f32,
}

impl Play {
    pub fn new(playlist_song_limit: i32, song_length_limit: f32) -> Self {
        Self {
            playlist_song_limit,
            song_length_limit,
        }
    }
}

#[async_trait]
impl CadencyCommand for Play {
    async fn execute<'a>(
        &self,
        ctx: &Context,
        command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let (search_payload, is_url, is_playlist) = {
            let query = self.arg_query(command);
            let (is_valid_url, is_playlist): (bool, bool) =
                Url::parse(&query).ok().map_or((false, false), |valid_url| {
                    let is_playlist: bool = valid_url
                        .query_pairs()
                        .find(|(key, _)| key == "list")
                        .is_some_and(|_| true);
                    (true, is_playlist)
                });
            (query, is_valid_url, is_playlist)
        };

        let (manager, call, guild_id) = utils::voice::join(ctx, command).await?;

        let response_builder = if is_playlist {
            let playlist_items =
                cadency_yt_playlist::fetch_playlist_songs(search_payload.clone()).unwrap();
            playlist_items
                .messages
                .iter()
                .for_each(|entry| debug!("🚧 Unable to parse song from playlist: {entry:?}",));
            let songs = playlist_items.data;
            let mut amount_added_playlist_songs = 0;
            let mut amount_total_added_playlist_duration = 0_f32;
            let mut skipped_by_limit = 0;
            let mut skipped_unavailable = 0;

            for song in songs {
                if amount_added_playlist_songs <= self.playlist_song_limit
                    && song.duration <= self.song_length_limit
                {
                    match utils::voice::add_song(ctx, call.clone(), song.url, true).await {
                        Ok((added_song_meta, _)) => {
                            amount_added_playlist_songs += 1;
                            amount_total_added_playlist_duration += song.duration;
                            debug!("➕ Added song '{:?}' from playlist", added_song_meta.title);
                        }
                        Err(err) => {
                            error!("❌ Failed to add song: {err}");
                            skipped_unavailable += 1;
                        }
                    }
                } else {
                    skipped_by_limit += 1;
                }
            }
            amount_total_added_playlist_duration /= 60_f32;
            {
                let mut handler = call.lock().await;
                handler.remove_all_global_events();
                handler.add_global_event(
                    Event::Periodic(std::time::Duration::from_secs(120), None),
                    InactiveHandler { guild_id, manager },
                );
            }

            let mut description = format!(
                "✅ **Added {} song{} to the queue**\n⏱️ **Total Duration:** {:.1} minutes",
                amount_added_playlist_songs,
                if amount_added_playlist_songs == 1 {
                    ""
                } else {
                    "s"
                },
                amount_total_added_playlist_duration
            );

            if skipped_by_limit > 0 {
                description.push_str(&format!(
                    "\n⚠️ **Skipped:** {} song{} (exceeded limits)",
                    skipped_by_limit,
                    if skipped_by_limit == 1 { "" } else { "s" }
                ));
            }

            if skipped_unavailable > 0 {
                description.push_str(&format!(
                    "\n🚫 **Unavailable:** {} song{} (removed or restricted)",
                    skipped_unavailable,
                    if skipped_unavailable == 1 { "" } else { "s" }
                ));
            }

            description.push_str(&format!("\n🎵 **Now Playing**"));

            let embed = serenity::builder::CreateEmbed::default()
                .title("📋 Playlist Added")
                .color(Colour::from_rgb(0, 255, 127)) // Spring green
                .description(description)
                .footer(serenity::all::CreateEmbedFooter::new(format!(
                    "Playlist limit: {} songs, {} seconds per song",
                    self.playlist_song_limit, self.song_length_limit as i32
                )));
            response_builder.embeds(vec![embed])
        } else {
            let (added_song_meta, _) = utils::voice::add_song(ctx, call.clone(), search_payload.clone(), is_url)
                .await
                .map_err(|err| {
                    let err_str = format!("{}", err);
                    error!("❌ Failed to add song to queue: {}", err);

                    let message = if err_str.contains("Video unavailable") || err_str.contains("not available") {
                        "❌ **Video is unavailable!**\n\nThis video may be private, deleted, or region-restricted."
                    } else {
                        "❌ **Couldn't add audio source to the queue!**\n\nPlease check the URL or search query."
                    };

                    CadencyError::Command {
                        message: message.to_string(),
                    }
                })?;
            {
                let mut handler = call.lock().await;
                handler.remove_all_global_events();
                handler.add_global_event(
                    Event::Periodic(std::time::Duration::from_secs(120), None),
                    InactiveHandler { guild_id, manager },
                );
            }

            let title = added_song_meta
                .title
                .as_ref()
                .map_or("Unknown Title", |title| title);
            let song_url = if is_url {
                search_payload.clone()
            } else {
                added_song_meta
                    .source_url
                    .as_ref()
                    .map_or("Unknown URL".to_string(), |url| url.to_owned())
            };

            let mut description = format!("🎵 **Title:** `{}`", title);

            if song_url != "Unknown URL" {
                description.push_str(&format!("\n🔗 **Source:** [View on YouTube]({})", song_url));
            }

            // Add duration if available
            if let Some(duration) = added_song_meta.duration {
                let minutes = duration.as_secs() / 60;
                let seconds = duration.as_secs() % 60;
                description.push_str(&format!("\n⏱️ **Duration:** {}:{:02}", minutes, seconds));
            }

            description.push_str("\n\n✅ **Added to queue and started playing!**");

            let embed = serenity::builder::CreateEmbed::default()
                .title("🎶 Song Added")
                .color(Colour::from_rgb(65, 105, 225)) // Royal blue
                .description(description);
            response_builder.embeds(vec![embed])
        };
        Ok(response_builder.build()?)
    }
}
