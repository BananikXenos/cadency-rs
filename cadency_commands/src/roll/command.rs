use super::dice::{RollDice, Throw};
use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use serenity::model::colour::Colour;

#[derive(CommandBaseline, Default)]
#[description = "Roll a dice of n sides"]
#[argument(
    name = "roll",
    description = "Dice(s) to roll. Only the following patterns are supported: `d6`, `2d6`, `2d6+1` or `2d6-1`",
    kind = "String"
)]
pub struct Roll {}

impl Roll {}

#[async_trait]
impl CadencyCommand for Roll {
    async fn execute<'a>(
        &self,
        _ctx: &Context,
        command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let throw_str = self.arg_roll(command);
        let throw = throw_str.parse::<Throw>()?;
        throw.validate()?;
        let roll = throw.roll();

        let description = format!(
            "🎲 **Roll:** `{}`\n🎯 **Result:** **{}**",
            throw_str, roll
        );

        let embed = serenity::builder::CreateEmbed::default()
            .title("🎲 Dice Roll")
            .color(Colour::from_rgb(138, 43, 226)) // Blue violet
            .description(description)
            .footer(serenity::all::CreateEmbedFooter::new(
                "Supported formats: d6, 2d6, 2d6+1, 2d6-1"
            ));
        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}