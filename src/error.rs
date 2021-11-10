use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlashCommandError {
    #[error("Unhandled slash command with name '{0}'")]
    UnhandledSlashCommand(String),
    #[error("{0}")]
    SerenityError(#[from] serenity::Error),
}
