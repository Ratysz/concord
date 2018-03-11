use protocol::*;
use protocol::serenity::prelude::*;
use protocol::serenity::model::prelude::*;

impl From<Message> for OmniMessage {
    fn from(msg: Message) -> Self {
        OmniMessage {
            channel: OmniChannel {
                discord: msg.channel_id,
            },
            text: msg.content,
        }
    }
}

pub struct Discord;

struct Handler {
    tx: Sender<OmniMessage>,
}

// TODO: re-implement self-consciousness.
impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        info!("[Discord] Sent message: {:?}", msg);
        if let Err(e) = self.tx.clone().send(OmniMessage::from(msg.clone())).wait() {
            error!("[Discord] Failed to transmit: {}", e);
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        info!(
            "[Discord] Connected as {}({}).",
            ready.user.name, ready.user.id
        );
    }
}

impl OmniProtocol for Discord {
    fn new(config: OmniConfig) -> OmniProtocolResult {
        info!("[Discord] Starting up.");
        let token = config.get::<String>("discord_token").unwrap();
        let (in_tx, in_rx) = channel::<OmniMessage>(10);
        let (out_tx, out_rx) = channel::<OmniMessage>(10);

        info!("[Discord] Configured, spawning threads.");
        let handle = thread::spawn(move || {
            info!("[Discord] Sender thread spawned.");
            let handle = thread::spawn(move || {
                info!("[Discord] Receiver thread spawned.");
                for message in in_rx.wait() {
                    info!("[Discord] Received message: {:?}", message);
                    if let Ok(msg) = message {
                        if let Err(e) = msg.channel.discord.say(format!("`{}`", msg.text)) {
                            error!("[Discord] Failed to say: {}", e);
                        }
                    }
                }
                info!("[Discord] Receiver thread done.");
            });

            match Client::new(&token, Handler { tx: out_tx }) {
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
        Ok(("discord", in_tx, out_rx, handle))
    }
}
