#![feature(type_alias_impl_trait)]

use uzh_biomed_bot::chat::*;
use uzh_biomed_bot::constant;
use uzh_biomed_bot::persistence::*;
use uzh_biomed_bot::scheduling::*;

use dotenv;
use std::error::Error;
use tbot::{
    prelude::*,
    types::keyboard::inline::{Button, ButtonKind},
};
type Context<T> = std::sync::Arc<tbot::contexts::Command<tbot::contexts::Text<T>>>;
type CallbackContext<T> = std::sync::Arc<tbot::contexts::DataCallback<T>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if dotenv::dotenv().is_err() {
        println!("No .env file found, reading config only from environment");
    }

    let _schedule_handle = schedule_maths();

    let mut bot = tbot::Bot::from_env("BOT_TOKEN").event_loop();
    bot.username("uzh_biomedicine_bot".to_owned());
    bot.start(handle_subscription);
    bot.command("subscribe", handle_subscription);
    bot.command("unsubscribe", handle_unsubscription);
    bot.command("links", handle_links);
    bot.data_callback(handle_callback);

    bot.polling().start().await.unwrap();
    Ok(())
}

async fn handle_subscription(context: Context<impl tbot::connectors::Connector>) {
    let chat = get_chat_from_context(&context).await;
    let chats = read_chats().expect("Failed to read chats");
    if chats
        .into_iter()
        .find(|compared_chat| compared_chat == &chat)
        .is_some()
    {
        context
                .send_message("You've already subscribed this chat to livestream announcements. You can unsubscribe again by using /unsubscribe")
                .call()
                .await
                .unwrap();
    } else {
        append_chat(chat).expect("Failed to append to chat");
        context
                .send_message("Successfully subscribed chat to livestream announcements. You can unsubscribe again by using /unsubscribe")
                .call()
                .await
                .unwrap();
    }
}

async fn get_chat_from_context(context: &Context<impl tbot::connectors::Connector>) -> Chat {
    let telegram_chat = context
        .get_chat()
        .call()
        .await
        .expect("Failed to retrieve chat");

    Chat {
        id: telegram_chat.id,
    }
}

async fn handle_unsubscription(context: Context<impl tbot::connectors::Connector>) {
    let chat = get_chat_from_context(&context).await;
    let removed_chat = remove_chat(chat).expect("Failed to read chats");

    if removed_chat.is_some() {
        context
                .send_message("You've successfully unsubscribed this chat from livestream announcements. You can subscribe again by using /subscribe")
                .call()
                .await
                .unwrap();
    } else {
        context
                .send_message("You are not subscribed to livestream announcements, so you can't unsubscribe from them. If you meant to subscribe, you can do so by using /subscribe")
                .call()
                .await
                .unwrap();
    }
}

async fn handle_links(context: Context<impl tbot::connectors::Connector>) {
    const KEYBOARD: &[&[Button]] = &[&[
        Button::new(
            "MAT 182",
            ButtonKind::CallbackData(constant::MATHS_CALLBACK),
        ),
        Button::new(
            "PHY 117",
            ButtonKind::CallbackData(constant::PHYSICS_CALLBACK),
        ),
    ]];

    context
        .send_message("Select the module you wish to see links for")
        .reply_markup(KEYBOARD)
        .call()
        .await
        .unwrap();
}

async fn handle_callback(context: CallbackContext<impl tbot::connectors::Connector>) {
    let message = match context.data.as_str() {
        constant::MATHS_CALLBACK => "You're cool too!",
        constant::PHYSICS_CALLBACK => "Thanks, I'm trying!",
        _ => "Are you trying to hack me?",
    };

    let call_result = context.notify(message).call().await;
    if let Err(err) = call_result {
        dbg!(err);
    }
}
