use cadency_core::{
    response::{Response, ResponseBuilder},
    CadencyCommand, CadencyError,
};
use num_bigint::BigUint;
use serenity::model::colour::Colour;
use serenity::{async_trait, client::Context, model::application::CommandInteraction};

#[derive(CommandBaseline, Default)]
#[description = "Calculate the nth number in the fibonacci sequence"]
#[argument(
    name = "number",
    description = "The number in the fibonacci sequence",
    kind = "Integer"
)]
pub struct Fib {}

impl Fib {
    fn calc(n: &i64) -> Result<BigUint, CadencyError> {
        if *n < 0 {
            return Err(CadencyError::Command {
                message: "❌ **The number must be non-negative**".to_string(),
            });
        }

        match *n {
            0 => Ok(BigUint::from(0u32)),
            1 => Ok(BigUint::from(1u32)),
            _ => {
                let (mut prev, mut curr) = (BigUint::from(0u32), BigUint::from(1u32));
                for _ in 2..=*n {
                    let next = &prev + &curr;
                    prev = curr;
                    curr = next;
                }
                Ok(curr)
            }
        }
    }
}

#[async_trait]
impl CadencyCommand for Fib {
    async fn execute<'a>(
        &self,
        _ctx: &Context,
        command: &'a mut CommandInteraction,
        response_builder: &'a mut ResponseBuilder,
    ) -> Result<Response, CadencyError> {
        let n = self.arg_number(command);
        let fib_value = Self::calc(&n)?;

        let description = format!("🔢 **Position:** {}\n📊 **Result:** `{}`", n, fib_value);

        let embed = serenity::builder::CreateEmbed::default()
            .title("🧮 Fibonacci Calculator")
            .color(Colour::from_rgb(30, 144, 255)) // Dodger blue
            .description(description);
        Ok(response_builder.embeds(vec![embed]).build()?)
    }
}
