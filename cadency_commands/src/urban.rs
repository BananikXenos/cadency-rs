use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::Context,
    model::application::CommandInteraction,
    model::colour::Colour,
};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct UrbanEntry {
    pub definition: String,
    pub permalink: String,
    pub thumbs_up: i64,
    pub author: String,
    pub word: String,
    pub defid: i64,
    pub written_on: String,
    pub example: String,
    pub thumbs_down: i64,
}

#[derive(serde::Deserialize, Debug)]
struct UrbanResult {
    pub list: Vec<UrbanEntry>,
}

#[derive(CommandBaseline, Default)]
#[description = "Searches the Urbandictionary for your query"]
#[deferred = true]
#[argument(name = "query", description = "Your search query", kind = "String")]
pub struct Urban {}

impl Urban {
    async fn request_urban_dictionary_entries(
        query: &str,
    ) -> Result<Vec<UrbanEntry>, reqwest::Error> {
        debug!("Requesting urban dictionary and deserialize json body");
        let url = format!("https://api.urbandictionary.com/v0/define?term={query}");
        Ok(reqwest::get(url).await?.json::<UrbanResult>().await?.list)
    }

    fn create_embed(urban_entries: Vec<UrbanEntry>) -> Vec<CreateEmbed> {
        let mut embeds: Vec<CreateEmbed> = Vec::new();

        // Helper closure to safely truncate strings to a byte limit
        // preserving valid UTF-8 boundaries.
        let safe_truncate = |text: &str, limit: usize| -> String {
            if text.len() <= limit {
                return text.to_string();
            }

            // We want to cut at 'limit - 3' to make room for "..."
            let target_len = limit - 3;

            let cut_index = text
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|&i| i <= target_len)
                .last()
                .unwrap_or(0);

            format!("{}...", &text[..cut_index])
        };

        for (index, urban) in urban_entries.iter().enumerate() {
            if index >= 3 {
                break;
            }

            let word = urban.word.replace(['[', ']'], "");
            let definition = urban.definition.replace(['[', ']'], "");
            let example = urban.example.replace(['[', ']'], "");

            // Apply the safe truncation
            let definition_display = safe_truncate(&definition, 1024);
            let example_display = safe_truncate(&example, 1024);

            let mut embed = CreateEmbed::default()
                .color(Colour::from_rgb(255, 215, 0)) // Gold
                .title(format!("📖 {}", word))
                .url(&urban.permalink)
                .field("📝 Definition", definition_display, false);

            if !example_display.is_empty() {
                embed = embed.field("💬 Example", example_display, false);
            }

            embed = embed.field(
                "📊 Rating",
                format!("👍 {} • 👎 {}", urban.thumbs_up, urban.thumbs_down),
                true,
            );

            embed = embed.field("✍️ Author", &urban.author, true);

            if index == 0 {
                embed = embed.footer(serenity::all::CreateEmbedFooter::new(format!(
                    "Showing top {} result{}",
                    urban_entries.len().min(3),
                    if urban_entries.len() > 1 { "s" } else { "" }
                )));
            }

            embeds.push(embed);
        }
        embeds
    }
}

#[async_trait]
impl CadencyCommand for Urban {
    async fn execute<'a>(
        &self,
        _ctx: &Context,
        command: &'a mut CommandInteraction,
        respone_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let query = self.arg_query(command);
        let urbans = Self::request_urban_dictionary_entries(&query)
            .await
            .map_err(|err| {
                error!("Failed to request urban dictionary entries : {:?}", err);
                CadencyError::Command {
                    message: "❌ **Failed to request urban dictionary**\n\nPlease try again later.".to_string(),
                }
            })?;
        let response_builder = if urbans.is_empty() {
            respone_builder.message(Some(format!("❌ **Nothing found for \"{}\"**\n\nTry a different search term!", query)))
        } else {
            respone_builder.embeds(Self::create_embed(urbans))
        };
        Ok(response_builder.build()?)
    }
}