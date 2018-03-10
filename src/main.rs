#![windows_subsystem = "windows"]
extern crate config;
extern crate futures;
#[macro_use]
extern crate log;
extern crate serenity;
extern crate simplelog;

mod commands;
mod prelude;
mod protocol;

use prelude::*;
use protocol::irc;

use futures::Future;
use futures::sync::mpsc::channel;
use serenity::prelude as serenity_client;
use serenity::model::channel::Message as SerMessage;
use serenity::model::gateway::Ready;
use simplelog::TermLogger;

use std::sync::Mutex;
use std::thread;

#[derive(Debug)]
pub struct OmniChannel {
    discord: serenity::model::id::ChannelId,
}

#[derive(Debug)]
pub struct OmniMessage {
    channel: OmniChannel,
    text: String,
}

impl From<SerMessage> for OmniMessage {
    fn from(msg: SerMessage) -> Self {
        OmniMessage {
            channel: OmniChannel {
                discord: msg.channel_id,
            },
            text: msg.content,
        }
    }
}

pub trait OmniProtocol {
    fn new(
        config: OmniConfig,
    ) -> Result<(&'static str, Sender<OmniMessage>, Receiver<OmniMessage>), &'static str>;
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
    TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();

    let mut config = config::Config::default();
    config.merge(config::File::with_name("config")).unwrap();

    let (discord_tx, discord_rx) = channel(10);

    /*thread::spawn(move || {
        for message in irc_rx.wait() {
            info!("[Discord] Received message: {:?}", message);
            if let Ok(_msg) = message {
                let msg = _msg as OmniMessage;
                if let Err(e) = msg.channel.discord.say(format!("`{}`", msg.text)) {
                    error!("[Discord] Failed to say: {}", e);
                }
            }
        }
    });*/

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
