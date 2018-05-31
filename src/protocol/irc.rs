use irc_crate::client::prelude as irc;
use protocol::*;

impl From<Message> for irc::Message {
    fn from(msg: Message) -> Self {
        unimplemented!();
        /*let (channel, text) = match msg.command.clone() {
            Command::PRIVMSG(chan, cont) => (
                chan,
                format!(
                    "{}: {}",
                    {
                        match msg.prefix {
                            Some(prefix) => prefix,
                            None => "NONE".to_string(),
                        }
                    },
                    cont
                ),
            ),
            _ => ("debug".to_string(), format!("{:?}", msg)),
        };
        CCMessage { channel, text }*/
    }
}

pub struct Irc;

impl Protocol for Irc {
    fn initialize(runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        unimplemented!();
        /*let irc_config = Config {
            nickname: Some("the-irc-crate".to_owned()),
            server: Some("irc.mozilla.org".to_owned()),
            channels: Some(vec!["#ratys-bot-test".to_owned()]),
            ..Config::default()
        };

        let (in_tx, in_rx) = channel::<CCMessage>(100);
        let (out_tx, out_rx) = channel::<CCMessage>(100);

        let handle = thread::spawn(move || {
            let mut reactor = IrcReactor::new().unwrap();
            let client = reactor.prepare_client_and_connect(&irc_config).unwrap();
            client.identify().unwrap();
            let client_clone = client.clone();
            reactor.inner_handle().spawn(in_rx.for_each(move |msg| {
                debug!("Received message: {:?}", msg);
                client_clone.send_privmsg(&msg.channel, &msg.text).unwrap();
                Ok(())
            }));
            reactor.register_client_with_handler(client, move |client, msg| {
                debug!("Sending message: {:?}", msg.clone());
                if msg.source_nickname() != Some(client.current_nickname()) {
                    if let Err(e) = out_tx.clone().send(CCMessage::from(msg)).wait() {
                        error!("IRC failed to transmit: {}", e);
                    }
                }
                Ok(())
            });
            reactor.run().unwrap();
        });

        Ok(CCProtocolHandles {
            protocol_tag: CCProtocolTag("irc"),
            sender: in_tx,
            receiver: out_rx,
            join_handle: handle,
        })*/
    }
}
