use crate::{
    builder::SlashCommandBuilder, error::SlashCommandError, SlashCommand, SlashCommandCallback,
};
use serenity::{
    async_trait,
    builder::{
        CreateApplicationCommand, CreateApplicationCommandPermissionsData,
        CreateApplicationCommands,
    },
    client::Context,
    model::{id::GuildId, interactions::application_command::ApplicationCommandInteraction},
    Result,
};
use std::collections::HashMap;
use tokio::sync::Mutex;

#[async_trait]
pub trait SlashCommandHandler: Sized + Send + Sync {
    async fn get_callback(&self, name: &str) -> Option<SlashCommandCallback>;

    async fn register_slash_command<T: SlashCommand>(&mut self) {
        T::register(self).await
    }

    async fn create_slash_command<F: Send>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut SlashCommandBuilder) -> &mut SlashCommandBuilder;

    async fn create_application_commands(&mut self, ctx: &Context) -> Result<()>;

    async fn create_application_command(
        &mut self,
        ctx: &Context,
        cmd: SlashCommandEntry,
    ) -> Result<()> {
        if cmd.guilds.is_some() {
            self.create_guild_command(ctx, cmd).await
        } else {
            self.create_global_command(ctx, cmd).await
        }
    }

    async fn create_guild_command(&mut self, ctx: &Context, cmd: SlashCommandEntry) -> Result<()> {
        for &guild_id in cmd.guilds.unwrap() {
            let guild_id = GuildId(guild_id);

            let create = cmd.create.clone();
            let guild_cmd = guild_id
                .create_application_command(ctx, move |c| {
                    c.0 = create.0;
                    c
                })
                .await?;

            let permissions = cmd.permissions.clone();
            guild_id
                .create_application_command_permission(ctx, guild_cmd.id, move |p| {
                    p.0 = permissions.0;
                    p
                })
                .await?;
        }
        Ok(())
    }

    async fn create_global_command(&mut self, ctx: &Context, cmd: SlashCommandEntry) -> Result<()>;

    async fn interaction_create(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> std::result::Result<(), SlashCommandError> {
        let command_name = interaction.data.name.as_str();
        match self.get_callback(command_name).await {
            Some(callback) => callback(ctx, interaction).await.map_err(Into::into),
            None => Err(SlashCommandError::UnhandledSlashCommand(
                command_name.to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlashCommandEntry {
    pub name: &'static str,
    pub guilds: Option<&'static [u64]>,
    pub create: CreateApplicationCommand,
    pub permissions: CreateApplicationCommandPermissionsData,
}

pub struct DefaultSlashCommandHandler {
    to_add: Option<Vec<SlashCommandEntry>>,
    globals: Option<CreateApplicationCommands>,
    callbacks: Mutex<HashMap<&'static str, SlashCommandCallback>>,
}

#[async_trait]
impl SlashCommandHandler for DefaultSlashCommandHandler {
    async fn get_callback(&self, name: &str) -> Option<SlashCommandCallback> {
        let callbacks = self.callbacks.lock().await;
        callbacks.get(name).copied()
    }

    async fn create_slash_command<F: Send>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut SlashCommandBuilder) -> &mut SlashCommandBuilder,
    {
        let mut builder = Default::default();
        f(&mut builder);

        let (entry, callback) = builder.build();
        {
            let mut callbacks = self.callbacks.lock().await;
            callbacks.insert(entry.name, callback);
        }
        self.to_add.as_mut().unwrap().push(entry);

        self
    }

    async fn create_application_commands(&mut self, ctx: &Context) -> Result<()> {
        let to_add = match self.to_add.take() {
            Some(t) => t,
            None => return Ok(()),
        };

        for cmd in to_add {
            self.create_application_command(ctx, cmd).await?;
        }

        Ok(())
    }

    async fn create_global_command(
        &mut self,
        _ctx: &Context,
        cmd: SlashCommandEntry,
    ) -> Result<()> {
        let globals = match self.globals.as_mut() {
            Some(g) => g,
            None => return Ok(()),
        };

        globals.add_application_command(cmd.create);

        Ok(())
    }
}

impl Default for DefaultSlashCommandHandler {
    fn default() -> Self {
        Self {
            to_add: Some(Default::default()),
            globals: Some(Default::default()),
            callbacks: Default::default(),
        }
    }
}
