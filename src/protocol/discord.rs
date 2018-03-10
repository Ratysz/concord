use protocol::*;
use protocol::serenity::prelude::*;
use protocol::serenity::model::prelude::*; //channel::Message;

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
        let token = config.get::<String>("discord_token").unwrap();

        let (in_tx, in_rx) = channel(10);
        let (out_tx, out_rx) = channel(10);

        let handle = thread::spawn(move || {
            let handle = thread::spawn(move || {
                let mut client = Client::new(&token, Handler { tx: out_tx })
                    .expect("[Discord] Failed to create client!");
                if let Err(e) = client.start() {
                    error!("[Discord] Client error: {:?}", e);
                }
            });

            for message in in_rx.wait() {
                info!("[Discord] Received message: {:?}", message);
                if let Ok(_msg) = message {
                    let msg = _msg as OmniMessage;
                    if let Err(e) = msg.channel.discord.say(format!("`{}`", msg.text)) {
                        error!("[Discord] Failed to say: {}", e);
                    }
                }
            }

            handle.join();
        });

        Ok(("discord", in_tx, out_rx, handle))
    }
}
