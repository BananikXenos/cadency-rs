use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use serenity::{async_trait, client::Context, model::application::CommandInteraction};

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
    fn validate_rating(rating: &str) -> Result<String, String> {
        let valid_ratings = ["safe", "suggestive", "borderline", "explicit"];
        let rating_lower = rating.to_lowercase();

        if valid_ratings.contains(&rating_lower.as_str()) {
            Ok(rating_lower)
        } else {
            Err(format!(
                "Invalid rating. Must be one of: {}",
                valid_ratings.join(", ")
            ))
        }
    }

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

    async fn request_anime_image_url(
        rating: String,
        tags: Option<Vec<String>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        debug!("Requesting nekosapi for a random anime image");
        let mut url = format!(
            "https://api.nekosapi.com/v4/images/random?limit=1&rating={}",
            rating
        );
        if let Some(tags) = tags {
            url.push_str(&format!("&tags={}", tags.join(",")));
        }
        let images: Vec<ImageEntry> = reqwest::get(&url).await?.json().await?;
        images
            .into_iter()
            .next()
            .map(|img| img.url)
            .ok_or_else(|| "empty response from nekosapi".into())
    }
}

#[async_trait]
impl CadencyCommand for Anime {
    async fn execute<'a>(
        &self,
        _ctx: &Context,
        _command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let rating = self
            .arg_rating(_command)
            .unwrap_or_else(|| "safe".to_string());
        let rating =
            Self::validate_rating(&rating).map_err(|err| CadencyError::Command { message: err })?;

        let tags = if let Some(tags_str) = self.arg_tags(_command) {
            Some(
                Self::validate_tags(&tags_str)
                    .map_err(|err| CadencyError::Command { message: err })?,
            )
        } else {
            None
        };

        let anime_url = Self::request_anime_image_url(rating, tags)
            .await
            .map_err(|err| {
                error!("{:?}", err);
                CadencyError::Command {
                    message: "**Nothing found with the given parameters**".to_string(),
                }
            })?;
        Ok(response_builder.message(Some(anime_url)).build()?)
    }
}
