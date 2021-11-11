mod builder;
mod error;
mod handler;
#[macro_use]
mod macros;

pub use builder::SlashCommandBuilder;
pub use error::SlashCommandError;
pub use handler::{DefaultSlashCommandHandler, SlashCommandEntry, SlashCommandHandler};

pub use proc_macro::slash_command;

use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateApplicationCommandPermissionsData},
    client::Context,
    futures::future::BoxFuture,
    model::interactions::application_command::ApplicationCommandInteraction,
    Result,
};

pub type SlashCommandCallback = for<'fut> fn(
    &'fut Context,
    &'fut ApplicationCommandInteraction,
) -> BoxFuture<'fut, serenity::Result<()>>;

#[async_trait]
pub trait SlashCommand: Send {
    const NAME: &'static str;
    const GUILDS: Option<&'static [u64]>;

    fn create(c: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    fn permissions(
        p: &mut CreateApplicationCommandPermissionsData,
    ) -> &mut CreateApplicationCommandPermissionsData {
        p
    }

    async fn callback(
        ctx: &'async_trait Context,
        interaction: &'async_trait ApplicationCommandInteraction,
    ) -> Result<()>;

    async fn register<H: SlashCommandHandler>(handler: &mut H) {
        handler
            .create_slash_command(|cmd| {
                cmd.name(Self::NAME)
                    .callback(Self::callback)
                    .create_application_command(|c| Self::create(c).name(Self::NAME))
                    .create_permissions(Self::permissions);
                if let Some(guilds) = Self::GUILDS {
                    cmd.guilds(guilds);
                }
                cmd
            })
            .await;
    }
}
