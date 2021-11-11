use serenity::{async_trait, model::prelude::*, prelude::*, Result};
use serenity_slash_command_framework::{
    DefaultSlashCommandHandler, SlashCommandBuilder, SlashCommandCallback, SlashCommandEntry,
    SlashCommandHandler,
};
use tracing::{error, info};

#[derive(Default)]
pub struct Handler {
    slash_commands: DefaultSlashCommandHandler,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _ctx: Context, guilds: Vec<GuildId>) {
        info!("Cache is ready! Found {} guilds.", guilds.len());
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::playing("/help")).await;
        info!("{} is connected!", ready.user.name);

        info!("Initializing slash commands...");
        if let Err(why) = self.create_application_commands(&ctx).await {
            error!("Error while creating slash commands: {}", why);
        }
        info!("Done initializing slash commands.");
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            if let Err(why) = self
                .slash_commands
                .interaction_create(&ctx, &interaction)
                .await
            {
                error!("Error while executing slash command: {}", why);
            }
        }
    }
}

#[async_trait]
impl SlashCommandHandler for Handler {
    async fn get_callback(&self, name: &str) -> Option<SlashCommandCallback> {
        self.slash_commands.get_callback(name).await
    }

    async fn create_slash_command<F: Send>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut SlashCommandBuilder) -> &mut SlashCommandBuilder,
    {
        self.slash_commands.create_slash_command(f).await;
        self
    }

    async fn create_application_commands(&self, ctx: &Context) -> Result<()> {
        self.slash_commands.create_application_commands(ctx).await
    }

    async fn create_global_command(&self, ctx: &Context, cmd: &SlashCommandEntry) -> Result<()> {
        self.slash_commands.create_global_command(ctx, cmd).await
    }
}
