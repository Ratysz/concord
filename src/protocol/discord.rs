use protocol::*;
use serenity;
use serenity::prelude::*;
use serenity::model::prelude::*;
use std::sync::RwLock;

impl From<Message> for OmniMessage {
    fn from(msg: Message) -> Self {
        OmniMessage {
            channel: msg.channel_id.to_string(),
            text: msg.content,
        }
    }
}

pub struct Discord;

struct Handler {
    tx: Sender<OmniMessage>,
    bot_user_id: RwLock<Option<serenity::model::id::UserId>>,
    // Needs interior mutability, is threaded, and unknown at init.
}

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if let Some(bot_id) = *self.bot_user_id.read().unwrap() {
            if msg.author.id != bot_id {
                debug!("Sending message: {:?}", &msg);
                if let Err(e) = self.tx.clone().send(OmniMessage::from(msg)).wait() {
                    error!("Discord failed to transmit: {}", e);
                }
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        info!(
            "Discord connected as {}({}).",
            ready.user.name, ready.user.id
        );
        let ref mut ref_bot_user_id = *self.bot_user_id.write().unwrap();
        *ref_bot_user_id = Some(ready.user.id);
    }
}

impl OmniProtocol for Discord {
    fn new(config: &OmniConfig) -> OmniProtocolResult {
        debug!("Starting up.");
        let token = config.get::<String>("discord_token").unwrap();
        let (in_tx, in_rx) = channel::<OmniMessage>(100);
        let (out_tx, out_rx) = channel::<OmniMessage>(100);

        debug!("Configured, spawning threads.");
        let handle = thread::spawn(move || {
            debug!("Sender thread spawned.");

            let handle = thread::spawn(move || {
                debug!("Receiver thread spawned.");
                for message in in_rx.wait() {
                    debug!("Received message: {:?}", message);
                    if let Ok(msg) = message {
                        if let Err(e) = ChannelId::from(msg.channel.parse::<u64>().unwrap())
                            .say(format!("`{}`", msg.text))
                        {
                            error!("Discord failed to say: {}", e);
                        }
                    }
                }
                debug!("Receiver thread done.");
            });

            match Client::new(
                &token,
                Handler {
                    tx: out_tx,
                    bot_user_id: RwLock::new(None), // No nulls - no problems.
                },
            ) {
                Ok(mut client) => if let Err(e) = client.start() {
                    error!("Discord client error: {}", e);
                },
                Err(e) => error!("Discord failed to create client: {}", e),
            }
            debug!("Sender thread done, joining.");

            handle.join().unwrap();
            debug!("Threads joined.");
        });
        debug!("Threads spawned.");

        Ok(("discord".to_string(), in_tx, out_rx, handle))
    }
}
