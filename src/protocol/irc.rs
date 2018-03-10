use protocol::*;
use protocol::irc_crate::client::prelude::*;

impl From<Message> for OmniMessage {
    fn from(msg: Message) -> Self {
        use protocol::serenity::model::id::ChannelId;
        OmniMessage {
            channel: OmniChannel {
                discord: ChannelId::from(409314585137512450 as u64),
            },
            text: format!("{:?}", msg),
        }
    }
}

pub struct Irc;

impl OmniProtocol for Irc {
    fn new(config: OmniConfig) -> OmniProtocolResult {
        let irc_config = Config {
            nickname: Some("the-irc-crate".to_owned()),
            server: Some("irc.mozilla.org".to_owned()),
            channels: Some(vec!["#ratys-bot-test".to_owned()]),
            ..Config::default()
        };

        let (in_tx, in_rx) = channel(10);
        let (out_tx, out_rx) = channel(10);

        let handle = thread::spawn(move || {
            let mut reactor = IrcReactor::new().unwrap();
            let client = reactor.prepare_client_and_connect(&irc_config).unwrap();
            client.identify().unwrap();
            reactor.inner_handle().spawn(in_rx.for_each(move |msg| {
                info!("[IRC] Received message: {:?}", msg);
                Ok(())
            }));
            reactor.register_client_with_handler(client, move |_client, msg| {
                info!("[IRC] Sent message: {:?}", msg.clone());
                if let Err(e) = out_tx.clone().send(OmniMessage::from(msg)).wait() {
                    error!("[IRC] Failed to transmit: {}", e);
                }
                Ok(())
            });
            reactor.run().unwrap();
        });

        Ok(("irc", in_tx, out_rx, handle))
    }
}