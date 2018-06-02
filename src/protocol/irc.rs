use failure;
use irc_crate;
use irc_crate::client::prelude::{self as irc, Client, ClientExt, IrcClient};
use protocol::*;

impl From<irc::Message> for Message {
    fn from(msg: irc::Message) -> Self {
        unimplemented!();
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

        /*let mut reactor = irc::IrcReactor::new().unwrap();
        let client_future = reactor.prepare_client(&irc_config).unwrap();

        runtime.spawn(client_future.map(|_| ()).map_err(|_| ()));*/

        let irc_config = irc::Config {
            nickname: Some("the-irc-crate".to_owned()),
            server: Some("irc.mozilla.org".to_owned()),
            channels: Some(vec!["#ratys-bot-test".to_owned()]),
            ..irc::Config::default()
        };

        /*let client_future = match irc::IrcClient::new_future(irc_config) {
            Ok(future) => future,
            Err(e) => {
                error!("irc init error: {}", &e);
                for cause in e.causes() {
                    error!("    {}", cause);
                }
                panic!();
            }
        }.map_err(|e| {
            error!("irc error: {}", &e);
            for cause in e.causes() {
                error!("    {}", cause);
            }
        });*/

        /*let client_future = irc::IrcReactor::new()
            .unwrap()
            .prepare_client(irc_config)
            .unwrap()
            .map_err(|e| {
                error!("irc error: {}", &e);
                for cause in e.causes() {
                    error!("    {}", cause);
                }
            });*/

        /*runtime.spawn(
            future::lazy(move || irc::IrcClient::new_future(irc_config))
                .and_then(|packed_client| {
                    let PackedIrcClient(client, future) = packed_client;
                    Ok((client, future))
                })
                .and_then(|(client, future)| {
                    info!("client: {:?}", client);
                    future
                }),
        );*/

        //let client_future = irc::IrcClient::new_future(irc_config).unwrap();

        runtime.spawn(
            irc::IrcClient::new_future(irc_config)
                .unwrap()
                .and_then(|(client, a_boi)| {
                    info!("client: {:?}", client);
                    let client_clone = client.clone();
                    a_boi
                        .futurize()
                        .join(client.stream().for_each(|message| {
                            info!("message: {:?}", message);
                            Ok(())
                        }))
                        .join(client.identify())
                        .and_then(|client| {
                            in_rx
                                .for_each(move |message| {
                                    trace!("Received message: {:?}", message);
                                    match message {
                                        Message::Control(command) => match command {
                                            Command::Shutdown => {
                                                debug!("Terminating.");
                                                return Err(());
                                            }
                                        },
                                        Message::Message {
                                            ref raw_contents, ..
                                        } => if let Err(e) = client_clone
                                            .send_privmsg("#ratys-bot-test", raw_contents)
                                        {
                                            error!("Failed to transmit: {}", e);
                                        },
                                    }
                                    Ok(())
                                })
                                .map_err(|_| irc_crate::error::IrcError::PingTimeout)
                        })
                })
                .map_err(|e| {
                    error!("irc error: {}", &e);
                    for cause in e.causes() {
                        error!("    {}", cause);
                    }
                }),
        );

        /*runtime.spawn(client_future.and_then(|packed_client| {
            let PackedIrcClient(client, future) = packed_client;
            info!("client: {:?}", client);
            let future = Box::new(future::ok(future))
                as Box<Future<Item = (), Error = irc_crate::error::IrcError>>;
            future
        }));*/

        /*let mut reactor = irc::IrcReactor::new().unwrap();
        let client = reactor.prepare_client_and_connect(&irc_config).unwrap();
        client.identify().unwrap();
        let client_clone = client.clone();
        reactor
            .inner_handle()
            .spawn(stream::iter_ok(in_rx).for_each(move |message| {
                trace!("Received message: {:?}", message);
                match message {
                    Message::Control(command) => match command {
                        Command::Shutdown => {
                            debug!("Terminating.");
                            return Err(());
                        }
                    },
                    Message::Message {
                        ref raw_contents, ..
                    } => if let Err(e) = client_clone.send_privmsg("#ratys-bot-test", raw_contents)
                    {
                        error!("Failed to transmit: {}", e);
                    },
                }
                Ok(())
            }));
        reactor.register_client_with_handler(client, move |client, message| {
            trace!("Sending message: {:?}", &message);
            if message.source_nickname() != Some(client.current_nickname()) {
                if let Err(e) = out_tx.send(Message::from(message)) {
                    error!("Failed to transmit: {}", e);
                }
            }
            Ok(())
        });

        reactor.run().unwrap();*/

        Ok(ProtocolHandles {
            protocol_tag: ProtocolTag("irc"),
            sender: in_tx,
            receiver: out_rx,
        })
    }
}
