use std::env;
use serenity::all::{ChannelId, CreateMessage, GuildId, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::builder::CreateAttachment;
use songbird::input::File;
use songbird::SerenityInit;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if !msg.content.to_lowercase().contains("yummers") {
            return;
        }

        let file = CreateAttachment::path("yummers.gif").await;
        let message = match file {
            Ok(f) => CreateMessage::new().content("yummers").add_file(f),
            Err(_) => CreateMessage::new().content("no yummer :("),
        };
        if msg.channel_id.send_message(&ctx.http, message).await.is_err() {
            return;
        }

        let guild_id = match msg.guild_id {
            Some(g) => g,
            None => return,
        };

        let voice_channel_id = match get_active_voice_channel_id(&ctx, guild_id, msg.author.id).await {
            Some(id) => id,
            None => return,
        };

        let manager = songbird::get(&ctx)
            .await
            .expect("Songbird not initialized")
            .clone();

        let handler_lock =  match manager.join(guild_id, voice_channel_id).await {
            Ok(call) => call,
            Err(err) => {
                eprintln!("Failed to join voice channel: {:?}", err);
                return;
            }
        };

        let handle = {
            let mut handler = handler_lock.lock().await;
            handler.enqueue_input(File::new("yummers.mp3").into()).await
        };
    }
}

async fn get_active_voice_channel_id(ctx: &Context, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
    ctx.http.get_user_voice_state(guild_id, user_id).await.ok()?.channel_id
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}