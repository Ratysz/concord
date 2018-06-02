use failure;
use irc_crate;
use irc_crate::client::prelude::{self as irc, Client, ClientExt, IrcClient};
use protocol::*;

impl From<irc::Message> for Message {
    fn from(msg: irc::Message) -> Self {
        let (channel, text) = match msg.command.clone() {
            irc::Command::PRIVMSG(chan, cont) => (
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
        let contents = vec![MessageFragment::Plain(text.clone())];
        Message::Message {
            author: AuthorTag("irc"),
            source: ChannelTag("irc"),
            raw_contents: text,
            contents,
        }
    }
}

pub struct Irc;

impl Protocol for Irc {
    fn initialize(self, runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        debug!("Initializing.");
        let (in_tx, in_rx) = unbounded();
        let (out_tx, out_rx) = unbounded();

        let irc_config = irc::Config {
            nickname: Some("the-irc-crate".to_owned()),
            server: Some("irc.mozilla.org".to_owned()),
            channels: Some(vec!["#ratys-bot-test".to_owned()]),
            ..irc::Config::default()
        };

        runtime.spawn(
            irc::IrcClient::new_future(irc_config)
                .unwrap()
                .and_then(move |(client, a_boi)| {
                    info!("client: {:?}", client);
                    a_boi
                        .futurize()
                        .join(client.stream().for_each(move |message| {
                            let message = Message::from(message);
                            trace!("Sending message: {:?}", &message);
                            spawn(out_tx.clone().send(message).then(|_| Ok(())));
                            Ok(())
                        }))
                        .join(future::lazy(|| {
                            client.identify();
                            in_rx
                                .for_each(move |message| {
                                    trace!("Received message: {:?}", message);
                                    match message {
                                        Message::Control(command) => match command {
                                            Command::Shutdown => {
                                                debug!("Terminating.");
                                                if let Err(e) = client.send_quit("Bye!") {
                                                    error!("Failed to transmit: {}", e);
                                                }
                                                return Err(());
                                            }
                                        },
                                        Message::Message {
                                            ref raw_contents, ..
                                        } => if let Err(e) =
                                            client.send_privmsg("#ratys-bot-test", raw_contents)
                                        {
                                            error!("Failed to transmit: {}", e);
                                        },
                                    }
                                    Ok(())
                                })
                                .map_err(|_| irc_crate::error::IrcError::PingTimeout)
                        }))
                })
                .map_err(|e| {
                    error!("irc error: {}", &e);
                    for cause in e.causes() {
                        error!("    {}", cause);
                    }
                })
                .then(|_| Ok(())),
        );

        Ok(ProtocolHandles {
            protocol_tag: ProtocolTag("irc"),
            sender: in_tx,
            receiver: out_rx,
        })
    }
}
