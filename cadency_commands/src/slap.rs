use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use serenity::model::colour::Colour;
use serenity::{async_trait, client::Context, model::application::CommandInteraction};
use std::num::NonZeroU64;

#[derive(CommandBaseline, Default)]
#[description = "Slap someone with a large trout!"]
#[argument(
    name = "target",
    description = "The user you want to slap",
    kind = "User"
)]
pub struct Slap {}

#[async_trait]
impl CadencyCommand for Slap {
    async fn execute<'a>(
        &self,
        _ctx: &Context,
        command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let target_id = self.arg_target(command);
        let invoker_id = command.user.id;
        let bot_id = command.application_id;

        let (title, description) = if target_id == invoker_id {
            (
                "🤔 Wait...",
                format!("**Why do you want to slap yourself, <@{}>?**", invoker_id),
            )
        } else if NonZeroU64::from(target_id) == NonZeroU64::from(bot_id) {
            (
                "🛡️ Nice try!",
                format!(
                    "**Nope!**\n<@{}> slaps <@{}> around a bit with a large trout!",
                    bot_id, invoker_id
                ),
            )
        } else {
            (
                "🖐️ Trout Slap!",
                format!(
                    "🐟 **<@{}>** slapped **<@{}>** with a big trout!\n\n*What did they do to deserve that?*",
                    invoker_id, target_id
                ),
            )
        };

        let embed = serenity::builder::CreateEmbed::default()
            .title(title)
            .color(Colour::from_rgb(255, 0, 127)) // Pink
            .description(description);

        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}
