use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use serenity::model::colour::Colour;
use serenity::{async_trait, client::Context, model::application::CommandInteraction};

#[derive(CommandBaseline, Default)]
#[description = "Play Ping-Pong"]
pub struct Ping {}

#[async_trait]
impl CadencyCommand for Ping {
    async fn execute<'a>(
        &self,
        _ctx: &Context,
        _command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let embed = serenity::builder::CreateEmbed::default()
            .title("🏓 Ping-Pong")
            .color(Colour::from_rgb(0, 255, 255)) // Aqua
            .description("🏓 **Pong!**\n\nBot is online and responding!");
        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}
