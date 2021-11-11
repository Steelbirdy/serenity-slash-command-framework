use serenity::{
    async_trait,
    builder::CreateInteractionResponseData,
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    Result,
};

#[async_trait]
pub trait ApplicationCommandInteractionExt {
    async fn create_channel_message<H, F>(&self, http: H, f: F) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync,
        F: Send + FnOnce(&mut CreateInteractionResponseData) -> &mut CreateInteractionResponseData;

    async fn defer_channel_message<H>(&self, http: H) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync;

    async fn update_message<H, F>(&self, http: H, f: F) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync,
        F: Send + FnOnce(&mut CreateInteractionResponseData) -> &mut CreateInteractionResponseData;

    async fn defer_update_message<H>(&self, http: H) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync;
}

#[async_trait]
impl ApplicationCommandInteractionExt for ApplicationCommandInteraction {
    async fn create_channel_message<H, F>(&self, http: H, f: F) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync,
        F: Send + FnOnce(&mut CreateInteractionResponseData) -> &mut CreateInteractionResponseData,
    {
        self.create_interaction_response(http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(f)
        })
        .await
    }

    async fn defer_channel_message<H>(&self, http: H) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync,
    {
        self.create_interaction_response(http, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    }

    async fn update_message<H, F>(&self, http: H, f: F) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync,
        F: Send + FnOnce(&mut CreateInteractionResponseData) -> &mut CreateInteractionResponseData,
    {
        self.create_interaction_response(http, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(f)
        })
        .await
    }

    async fn defer_update_message<H>(&self, http: H) -> Result<()>
    where
        H: AsRef<Http> + Send + Sync,
    {
        self.create_interaction_response(http, |r| {
            r.kind(InteractionResponseType::DeferredUpdateMessage)
        })
        .await
    }
}
