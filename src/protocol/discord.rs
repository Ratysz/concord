use protocol::*;
use serenity::model::id as s_id;
use serenity::model::prelude as s_model;
use serenity::prelude as s;
use std::sync::RwLock;

impl From<Message> for s_model::Message {
    fn from(msg: Message) -> Self {
        unimplemented!();
        /*CCMessage {
            channel: msg.channel_id.to_string(),
            text: msg.content,
        }*/
    }
}

pub struct Discord;

struct Handler {
    tx: Sender<Message>,
    bot_user_id: RwLock<Option<s_id::UserId>>,
    // Needs interior mutability, is threaded, and unknown at init.
}

impl s::EventHandler for Handler {
    fn message(&self, _: s::Context, msg: s_model::Message) {
        unimplemented!();
        /*if let Some(bot_id) = *self.bot_user_id.read().unwrap() {
            if msg.author.id != bot_id {
                trace!("Sending message: {:?}", &msg);
                if let Err(e) = self.tx.clone().send(CCMessage::from(msg)).wait() {
                    error!("Discord failed to transmit: {}", e);
                }
            }
        }*/
    }

    fn ready(&self, _: s::Context, ready: s_model::Ready) {
        unimplemented!();
        /*info!(
            "Discord connected as {}({}).",
            ready.user.name, ready.user.id
        );
        let ref mut ref_bot_user_id = *self.bot_user_id.write().unwrap();
        *ref_bot_user_id = Some(ready.user.id);*/
    }
}

impl Protocol for Discord {
    fn initialize(runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        unimplemented!();
        /*trace!("Starting up.");
        let token = config.get::<String>("discord_token").unwrap();
        let (in_tx, in_rx) = channel::<CCMessage>(100);
        let (out_tx, out_rx) = channel::<CCMessage>(100);

        trace!("Configured, spawning threads.");
        let handle = thread::spawn(move || {
            trace!("Sender thread spawned.");

            let handle = thread::spawn(move || {
                trace!("Receiver thread spawned.");
                for message in in_rx.wait() {
                    trace!("Received message: {:?}", message);
                    if let Ok(msg) = message {
                        if let Err(e) = ChannelId::from(msg.channel.parse::<u64>().unwrap())
                            .say(format!("`{}`", msg.text))
                        {
                            error!("Discord failed to say: {}", e);
                        }
                    }
                }
                trace!("Receiver thread done.");
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
            trace!("Sender thread done, joining.");

            handle.join().unwrap();
            trace!("Threads joined.");
        });
        trace!("Threads spawned.");

        Ok(CCProtocolHandles {
            protocol_tag: CCProtocolTag("discord"),
            sender: in_tx,
            receiver: out_rx,
            join_handle: handle,
        })*/
    }
}
