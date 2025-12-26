// rust
use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use serenity::all::CreateEmbedFooter;
use serenity::{
    async_trait, client::Context, model::application::CommandInteraction, model::channel::Channel,
    model::colour::Colour,
};
use std::str::FromStr;

const TAG_DISPLAY_LIMIT: usize = 10;

#[derive(Default, CommandBaseline)]
#[description = "Send a random anime image"]
#[argument(
    name = "rating",
    description = "Content rating filter (safe, suggestive, borderline, explicit)",
    required = false,
    kind = "String"
)]
#[allow(clippy::duplicated_attributes)]
#[argument(
    name = "tags",
    description = "Tags to filter images by (comma separated)",
    required = false,
    kind = "String"
)]
pub struct Anime {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rating {
    Safe,
    Suggestive,
    Borderline,
    Explicit,
}

impl Rating {
    fn is_nsfw(&self) -> bool {
        !matches!(self, Rating::Safe)
    }

    fn as_str(&self) -> &'static str {
        match self {
            Rating::Safe => "safe",
            Rating::Suggestive => "suggestive",
            Rating::Borderline => "borderline",
            Rating::Explicit => "explicit",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            Rating::Safe => "✅",
            Rating::Suggestive => "⚠️",
            Rating::Borderline => "🔞",
            Rating::Explicit => "🔞",
        }
    }
}

impl FromStr for Rating {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "safe" => Ok(Rating::Safe),
            "suggestive" => Ok(Rating::Suggestive),
            "borderline" => Ok(Rating::Borderline),
            "explicit" => Ok(Rating::Explicit),
            other => Err(format!(
                "Invalid rating '{}'. Must be one of: safe, suggestive, borderline, explicit",
                other
            )),
        }
    }
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
struct ImageEntry {
    id: u64,
    url: String,
    rating: String,
    color_dominant: Vec<u8>,
    color_palette: Vec<Vec<u8>>,
    artist_name: Option<String>,
    tags: Vec<String>,
    source_url: Option<String>,
}

impl Anime {
    fn validate_tags(tags_str: &str) -> Result<Vec<String>, String> {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tags.is_empty() {
            return Err("Tags cannot be empty".to_string());
        }

        for tag in &tags {
            if !tag
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(format!("Invalid tag format: '{}'", tag));
            }
        }

        Ok(tags)
    }

    async fn is_channel_nsfw(ctx: &Context, command: &CommandInteraction) -> bool {
        let Ok(channel) = command.channel_id.to_channel(ctx).await else {
            return false;
        };

        match channel {
            Channel::Guild(ch) => ch.nsfw,
            Channel::Private(_) => true,
            _ => false,
        }
    }

    async fn request_anime_image(
        rating: Rating,
        tags: Option<Vec<String>>,
    ) -> Result<ImageEntry, Box<dyn std::error::Error>> {
        debug!("Requesting nekosapi for a random anime image");
        let mut url = format!(
            "https://api.nekosapi.com/v4/images/random?limit=1&rating={}",
            rating.as_str()
        );
        if let Some(tags) = tags {
            url.push_str(&format!("&tags={}", tags.join(",")));
        }
        let images: Vec<ImageEntry> = reqwest::get(&url).await?.json().await?;
        images
            .into_iter()
            .next()
            .ok_or_else(|| "empty response from nekosapi".into())
    }
}

#[async_trait]
impl CadencyCommand for Anime {
    async fn execute<'a>(
        &self,
        ctx: &Context,
        command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let rating_str = self
            .arg_rating(command)
            .unwrap_or_else(|| "safe".to_string());
        let rating: Rating = rating_str
            .parse()
            .map_err(|err: String| CadencyError::Command { message: err })?;

        let tags = if let Some(tags_str) = self.arg_tags(command) {
            Some(
                Self::validate_tags(&tags_str)
                    .map_err(|err| CadencyError::Command { message: err })?,
            )
        } else {
            None
        };

        if rating.is_nsfw() && !Self::is_channel_nsfw(ctx, command).await {
            return Err(CadencyError::Command {
                message: "**NSFW content is not allowed in this channel**".to_string(),
            });
        }

        let image = Self::request_anime_image(rating, tags.clone())
            .await
            .map_err(|err| {
                error!("{:?}", err);
                CadencyError::Command {
                    message: "**Nothing found with the given parameters**".to_string(),
                }
            })?;

        // Build the embed description
        let mut description = String::new();

        // Artist information
        if let Some(artist) = &image.artist_name {
            description.push_str(&format!("🎨 **Artist:** {}\n", artist));
        }

        // Rating with emoji
        description.push_str(&format!(
            "{} **Rating:** {}\n",
            rating.emoji(),
            rating.as_str()
        ));

        // Tags
        if !image.tags.is_empty() {
            let tags_display = image
                .tags
                .iter()
                .take(TAG_DISPLAY_LIMIT) // Limit to first 10 tags to avoid too long descriptions
                .map(|t| format!("`{}`", t))
                .collect::<Vec<_>>()
                .join(", ");
            description.push_str(&format!("🏷️ **Tags:** {}", tags_display));
            if image.tags.len() > TAG_DISPLAY_LIMIT {
                description.push_str(&format!(
                    " *+{} more*",
                    image.tags.len() - TAG_DISPLAY_LIMIT
                ));
            }
            description.push('\n');
        }

        // Source link
        if let Some(source) = &image.source_url {
            description.push_str(&format!("🔗 **Source:** [View Original]({})\n", source));
        }

        // Image ID
        description.push_str(&format!("\n💾 Image ID: `{}`", image.id));

        let footer = CreateEmbedFooter::new("Powered by nekosapi.com")
            .icon_url("https://nekosapi.com/branding/logo/logo.png");

        let embed = serenity::builder::CreateEmbed::default()
            .title("🌸 Random Anime Image")
            .description(description)
            .color(Colour::from_rgb(255, 105, 180)) // Hot pink
            .image(image.url)
            .footer(footer);

        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}
