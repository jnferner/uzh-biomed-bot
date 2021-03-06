#![feature(type_alias_impl_trait)]

use uzh_biomed_bot::chat::*;
use uzh_biomed_bot::constant;
use uzh_biomed_bot::persistence::*;
use uzh_biomed_bot::scheduling::*;

use dotenv;
use std::error::Error;
use tbot::types::parameters::Text as ParseMode;
use tbot::{
    markup::*,
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
    const KEYBOARD: &[&[Button]] = &[
        &[
            Button::new(
                "UZH Websites",
                ButtonKind::CallbackData(constant::callback_token::UZH_WEBSITES),
            ),
            Button::new(
                "OLAT",
                ButtonKind::CallbackData(constant::callback_token::OLAT),
            ),
        ],
        &[
            Button::new(
                "MAT 183",
                ButtonKind::CallbackData(constant::callback_token::MAT_183),
            ),
            Button::new(
                "PHY 127",
                ButtonKind::CallbackData(constant::callback_token::PHY_127),
            ),
        ],
        &[Button::new(
            "Discord",
            ButtonKind::CallbackData(constant::callback_token::DISCORD),
        )],
    ];

    context
        .send_message("Select the module you wish to see links for")
        .reply_markup(KEYBOARD)
        .call()
        .await
        .unwrap();
}

async fn handle_callback(context: CallbackContext<impl tbot::connectors::Connector>) {
    let message = match context.data.as_str() {
        constant::callback_token::UZH_WEBSITES => markdown_v2((
            "The following UZH websites are relevant:\n- ",
            link("Homepage", "https://www.uzh.ch/de.html"),
            "\n- ",
            link("Webmail", "https://webmail.uzh.ch/"),
            "\n- ",
            link(
                "Launchpad",
                "https://studentservices.uzh.ch/uzh/launchpad/#Shell-home",
            ),
            "\n- ",
            link("Module Booking", "https://studentservices.uzh.ch/mb"),
            "\n- ",
            link(
                "Swisscovery",
                "https://swisscovery.slsp.ch/discovery/search?vid=41SLSP_UZB:VU1_UNION&lang=en",
            ),
        ))
        .to_string(),
        constant::callback_token::OLAT => markdown_v2(link(
            "OLAT",
            "https://lms.uzh.ch/auth/MyCoursesSite/0/Favorits/0",
        ))
        .to_string(),
        constant::callback_token::MAT_183 => markdown_v2((
            "The following links are important for MAT 183:\n- ",
            link(
                "OLAT",
                "https://lms.uzh.ch/auth/RepositoryEntry/16974184862/CourseNode/103233511448483",
            ),
            "\n- ",
            link("Website", "https://www.math.uzh.ch/mat183.1"),
            "\n- ",
            link(
                "Exercises",
                "https://w3.math.uzh.ch/my/index.php?id=lecture",
            ),
            "\n- ",
            link(
                "Slack Forum",
                "https://app.slack.com/client/T01LQ47LN3H/D01NUBXNCDR"
            )
        ))
        .to_string(),
        constant::callback_token::PHY_127 => markdown_v2((
            "The following links are important for PHY 127:\n- ",
            link(
                "OLAT",
                "https://lms.uzh.ch/auth/RepositoryEntry/16955310089/CourseNode/103233523024807",
            ),
            "\n- ",
            link(
                "Website",
                "https://www.physik.uzh.ch/de/lehre/PHY127/FS2021.html",
            ),
        ))
        .to_string(),
        constant::callback_token::DISCORD => markdown_v2((
            "The following Discord servers are used by students:\n- ",
            link("Biomed Erstis", "https://discord.gg/kNhWwUGt8a"),
            "\n- ",
            link("BIUZ Biomedizin Server", "https://discord.gg/Dt454GHdDE"),
            "\n- ",
            link("UZH Students", "https://discord.gg/XJU44tdZr3"),
        ))
        .to_string(),
        _ => panic!("Invalid callback"),
    };

    let chat_id = if let tbot::types::callback::query::Origin::Message(message) = &context.origin {
        message.chat.id
    } else {
        return;
    };

    let call_result = context
        .bot
        .send_message(chat_id, ParseMode::markdown_v2(&message))
        .call()
        .await;

    if let Err(err) = call_result {
        dbg!(err);
    }
}
