#![windows_subsystem = "windows"]
extern crate config;
extern crate futures;
extern crate irc;
#[macro_use]
extern crate log;
extern crate serenity;
extern crate simplelog;

use futures::{Future, Sink, Stream};
use futures::sync::mpsc::{channel, Sender};
use irc::client::prelude as irc_client;
use irc::client::prelude::ClientExt;
use irc::client::prelude::Message as IrcMessage;
use irc::client::prelude::Command as IrcCommand;
use serenity::prelude as serenity_client;
use serenity::model::channel::Message as SerMessage;
use serenity::model::gateway::Ready;
use simplelog::TermLogger;

use std::sync::Mutex;
use std::thread;

mod commands;

#[derive(Debug)]
struct OmniMessage {
    discord_channel: serenity::model::id::ChannelId,
    text: String,
}

impl From<SerMessage> for OmniMessage {
    fn from(msg: SerMessage) -> Self {
        OmniMessage {
            discord_channel: msg.channel_id,
            text: msg.content,
        }
    }
}

impl From<IrcMessage> for OmniMessage {
    fn from(msg: IrcMessage) -> Self {
        OmniMessage {
            discord_channel: serenity::model::id::ChannelId::from(409314585137512450 as u64),
            text: format!("{:?}", msg),
        }
    }
}

struct Handler {
    tx: Sender<OmniMessage>,
    bot_user_id: Mutex<Option<serenity::model::id::UserId>>,
    // Has to be Mutex<Option<T>>: needs interior mutability, is threaded, and unknown at init.
}

impl serenity_client::EventHandler for Handler {
    fn message(&self, _: serenity_client::Context, msg: SerMessage) {
        info!("[Discord] Sent message: {:?}", msg);
        let bot_id = {
            let ref mut locked = *self.bot_user_id.lock().unwrap();
            match *locked {
                Some(id) => id.clone(),
                None => {
                    // (Useless?) recovery attempt.
                    // There should be no overhead, if it's never called.
                    let id = serenity::http::get_current_user().unwrap().id;
                    *locked = Some(id.clone());
                    id
                }
            }
            // This is where MutexGuard should be dropped, unlocking the Mutex.
        };
        if msg.author.id != bot_id {
            if let Err(e) = self.tx.clone().send(OmniMessage::from(msg.clone())).wait() {
                error!("[Discord] Failed to transmit: {}", e);
            }
        }
    }

    fn ready(&self, _: serenity_client::Context, ready: Ready) {
        info!(
            "[Discord] Connected as {}({}), locking mutex.",
            ready.user.name, ready.user.id
        );
        let ref mut ref_user_id = *self.bot_user_id.lock().unwrap();
        *ref_user_id = Some(ready.user.id);
        info!("[Discord] UserID written, unlocking mutex.");
    }
}

fn main() {
    let mut config = config::Config::default();
    config.merge(config::File::with_name("config")).unwrap();

    TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();

    let (irc_tx, irc_rx) = channel(10);
    let (discord_tx, discord_rx) = channel(10);

    /*thread::spawn(move || {
        for msg in discord_rx.wait() {
            info!("[Discord] Transmitted message: {:?}", msg);
        }
    });*/

    thread::spawn(move || {
        for message in irc_rx.wait() {
            info!("[Discord] Received message: {:?}", message);
            if let Ok(_msg) = message {
                let msg = _msg as OmniMessage;
                if let Err(e) = msg.discord_channel.say(format!("`{}`", msg.text)) {
                    error!("[Discord] Failed to say: {}", e);
                }
            }
        }
    });

    let irc_config = irc_client::Config {
        nickname: Some("the-irc-crate".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#ratys-bot-test".to_owned()]),
        ..irc_client::Config::default()
    };

    thread::spawn(move || {
        let mut reactor = irc_client::IrcReactor::new().unwrap();
        let client = reactor.prepare_client_and_connect(&irc_config).unwrap();
        client.identify().unwrap();
        reactor
            .inner_handle()
            .spawn(discord_rx.for_each(move |msg| {
                info!("[IRC] Received message: {:?}", msg);
                Ok(())
            }));
        reactor.register_client_with_handler(client, move |_client, msg| {
            info!("[IRC] Sent message: {:?}", msg.clone());
            if let Err(e) = irc_tx.clone().send(OmniMessage::from(msg)).wait() {
                error!("[IRC] Failed to transmit: {}", e);
            }
            Ok(())
        });
        reactor.run().unwrap();
    });

    let token = config.get::<String>("discord_token").unwrap();
    let mut client = serenity_client::Client::new(
        &token,
        Handler {
            tx: discord_tx,
            bot_user_id: Mutex::new(None), // No nulls - no problems.
        },
    ).expect("[Discord] Failed to create client!");
    if let Err(e) = client.start() {
        error!("[Discord] Client error: {:?}", e);
    }
}
