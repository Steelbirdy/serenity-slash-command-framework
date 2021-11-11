use serenity::client::Context;
use serenity::model::interactions::{
    application_command::ApplicationCommandInteraction, InteractionResponseType,
};
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity_slash_command_framework::{slash_command, slash_command_permissions, slash_command_options};

slash_command_permissions!(owner: true for User(301106820834131969));
slash_command_options!(yes_no: |o| {
    o.kind(ApplicationCommandOptionType::String)
        .name("yes_no")
        .description("Confirm that you want to shut down the bot")
        .add_string_choice("yes", "y")
        .add_string_choice("no", "n")
});

#[slash_command]
#[description = "Shuts down the bot"]
#[guild(765314921151332464)]
#[permission(owner)]
#[option(yes_no)]
#[default_permission(false)]
pub async fn quit(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> serenity::Result<()> {
    let data = ctx.data.read().await;

    let yn = interaction.data.options.get(0);
    let confirm_shutdown = yn
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str().map(|s| s == "y"))
        .unwrap_or_default();

    if !confirm_shutdown {
        let _ = interaction
            .create_interaction_response(ctx, |i| {
                i.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| d.content("Please confirm shutdown."))
            })
            .await;
        return Ok(());
    }

    if let Some(manager) = data.get::<crate::ShardManagerContainer>() {
        let _ = interaction
            .create_interaction_response(ctx, |i| {
                i.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| d.content("Shutting down!"))
            })
            .await;
        manager.lock().await.shutdown_all().await;
    } else {
        let _ = interaction
            .create_interaction_response(ctx, |i| {
                i.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.content("There was a problem reaching the shard manager.")
                    })
            })
            .await;
    };

    Ok(())
}
