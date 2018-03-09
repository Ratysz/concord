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
use serenity::prelude as serenity_client;
use serenity::model::channel::Message as SerMessage;
use serenity::model::gateway::Ready;
use simplelog::TermLogger;

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
            text: msg.to_string(),
        }
    }
}

struct Handler {
    tx: Sender<OmniMessage>,
}

impl serenity_client::EventHandler for Handler {
    fn message(&self, _: serenity_client::Context, msg: SerMessage) {
        info!("{:?}", msg);
        if let Err(e) = self.tx.clone().send(OmniMessage::from(msg.clone())).wait() {
            error!("Failed to send from Discord thread: {}", e);
        }
    }

    fn ready(&self, _: serenity_client::Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }
}

fn main() {
    let mut config = config::Config::default();
    config.merge(config::File::with_name("config")).unwrap();

    TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();

    let (irc_tx, irc_rx) = channel(10);
    let (discord_tx, discord_rx) = channel(10);

    thread::spawn(move || {
        for msg in discord_rx.wait() {
            info!("Discord_rx message: {:?}", msg);
        }
    });

    thread::spawn(move || {
        for message in irc_rx.wait() {
            info!("IRC_rx message: {:?}", message);
            if let Ok(_msg) = message {
                let msg = _msg as OmniMessage;
                if let Err(e) = msg.discord_channel.say(format!("IRC_rx: {:?}", msg.text)) {
                    error!("Failed to post in Discord channel");
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

        reactor.register_client_with_handler(client, move |_client, msg| {
            info!("{}", msg);
            if let Err(e) = irc_tx.clone().send(OmniMessage::from(msg.clone())).wait() {
                error!("Failed to send from IRC thread: {}", e);
            }
            Ok(())
        });
        reactor.run().unwrap();
    });

    let token = config.get::<String>("discord_token").unwrap();
    let mut client = serenity_client::Client::new(&token, Handler { tx: discord_tx })
        .expect("Err creating client");
    if let Err(e) = client.start() {
        error!("Discord client error: {:?}", e);
    }

    info!("We're at the end now.");
}
