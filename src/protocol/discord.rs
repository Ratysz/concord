use protocol::*;
use protocol::serenity::prelude::*;
use protocol::serenity::model::prelude::*;
use std::sync::Mutex;

impl From<Message> for OmniMessage {
    fn from(msg: Message) -> Self {
        OmniMessage {
            channel: "409314585137512450",
            text: msg.content,
        }
    }
}

pub struct Discord;
const PROTOCOL_TAG: ProtocolTag = "discord";

struct Handler {
    tx: Sender<OmniMessage>,
    bot_user_id: Mutex<Option<serenity::model::id::UserId>>,
    // Has to be Mutex<Option<T>>: needs interior mutability, is threaded, and unknown at init.
}

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
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
            info!("[Discord] Sending message: {:?}", msg.clone());
            if let Err(e) = self.tx.clone().send(OmniMessage::from(msg)).wait() {
                error!("[Discord] Failed to transmit: {}", e);
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        info!(
            "[Discord] Connected as {}({}).",
            ready.user.name, ready.user.id
        );
        let ref mut ref_bot_user_id = *self.bot_user_id.lock().unwrap();
        *ref_bot_user_id = Some(ready.user.id);
    }
}

impl OmniProtocol for Discord {
    fn new(config: &OmniConfig) -> OmniProtocolResult {
        info!("[Discord] Starting up.");
        let token = config.get::<String>("discord_token").unwrap();
        let (in_tx, in_rx) = channel::<OmniMessage>(100);
        let (out_tx, out_rx) = channel::<OmniMessage>(100);

        info!("[Discord] Configured, spawning threads.");
        let handle = thread::spawn(move || {
            info!("[Discord] Sender thread spawned.");

            let handle = thread::spawn(move || {
                info!("[Discord] Receiver thread spawned.");
                for message in in_rx.wait() {
                    info!("[Discord] Received message: {:?}", message);
                    if let Ok(msg) = message {
                        if let Err(e) = ChannelId::from(msg.channel.parse::<u64>().unwrap())
                            .say(format!("`{}`", msg.text))
                        {
                            error!("[Discord] Failed to say: {}", e);
                        }
                    }
                }
                info!("[Discord] Receiver thread done.");
            });

            match Client::new(
                &token,
                Handler {
                    tx: out_tx,
                    bot_user_id: Mutex::new(None), // No nulls - no problems.
                },
            ) {
                Ok(mut client) => if let Err(e) = client.start() {
                    error!("[Discord] Client error: {}", e);
                },
                Err(e) => error!("[Discord] Failed to create client: {}", e),
            }
            info!("[Discord] Sender thread done, joining.");

            handle.join().unwrap();
            info!("[Discord] Threads joined.");
        });

        info!("[Discord] Threads spawned.");

        Ok((PROTOCOL_TAG, in_tx, out_rx, handle))
    }
}
