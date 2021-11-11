use serenity::client::Context;
use serenity::model::interactions::{
    application_command::ApplicationCommandInteraction, InteractionResponseType,
};
use serenity_slash_command_framework::{slash_command, slash_command_permissions};

slash_command_permissions!(owner: true for User(301106820834131969));

#[slash_command]
#[description = "Shuts down the bot"]
#[guild(765314921151332464)]
#[permission(owner)]
#[default_permission(false)]
pub async fn quit(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> serenity::Result<()> {
    let data = ctx.data.read().await;

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
