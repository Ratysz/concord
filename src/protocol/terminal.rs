use protocol::*;
use std::{thread, time};

impl From<String> for CCMessage {
    fn from(text: String) -> Self {
        let mut contents = Vec::new();
        if text.contains("shutdown") {
            contents.push(CCMessageFragment::Command(CCCommand::Shutdown));
        }
        CCMessage::Message {
            author: CCAuthorTag("term"),
            source_channel: CCChannelTag("term"),
            raw_contents: text,
            contents,
        }
        /*let mut vec: Vec<&str> = text.splitn(2, "|").collect();
        if let (Some(text), Some(channel)) = (vec.pop(), vec.pop()) {
            let msg = CCMessage {
                channel: channel.to_string(),
                text: text.to_string(),
            };
            debug!("Formed message: {:?}", msg);
            msg
        } else {
            warn!("Malformed message; proper syntax: channel|message");
            CCMessage {
                channel: "debug".to_string(),
                text: format!("Malformed message sent via terminal: {}", text).to_string(),
            }
        }*/
    }
}

pub struct Terminal;

impl CCProtocol for Terminal {
    fn initialize(runtime: &mut Runtime) -> CCResult<CCProtocolHandles> {
        trace!("Starting up.");
        let (in_tx, in_rx) = channel::<CCMessage>();
        let (out_tx, out_rx) = channel::<CCMessage>();
        let (intra_tx, intra_rx) = channel::<()>();

        runtime.spawn(future::loop_fn((in_rx, intra_tx), |(in_rx, intra_tx)| {
            //blocking(|| {
            if let Ok(message) = in_rx.recv_timeout(time::Duration::from_secs(1)) {
                match message {
                    CCMessage::Control(command) => match command {
                        Shutdown => {
                            intra_tx.send(());
                            trace!("Receiver task done.");
                            return Ok(future::Loop::Break(()));
                        }
                        _ => {
                            warn!("Unhandled command {:?}!", command);
                        }
                    },
                    CCMessage::Message { raw_contents, .. } => {
                        println!("{:?}", raw_contents);
                    }
                }
            }
            //});
            Ok(future::Loop::Continue((in_rx, intra_tx)))
        }));

        runtime.spawn(
            future::lazy(|| {
                let (term_tx, term_rx) = channel::<String>();
                trace!("Starting terminal input thread.");
                thread::spawn(move || loop {
                    let line: String = read!("{}\n");
                    term_tx.send(line);
                });
                Ok(term_rx)
            }).and_then(|term_rx| {
                future::loop_fn(
                    (out_tx, intra_rx, term_rx),
                    |(out_tx, intra_rx, term_rx)| {
                        if intra_rx.try_recv().is_err() {
                            //blocking(|| {
                            if let Ok(line) = term_rx.recv_timeout(time::Duration::from_secs(1)) {
                                trace!("Sending message: {:?}", &line);
                                out_tx.send(CCMessage::from(line)).unwrap();
                            }
                            //});
                            Ok(future::Loop::Continue((out_tx, intra_rx, term_rx)))
                        } else {
                            trace!("Sender task done.");
                            Ok(future::Loop::Break(()))
                        }
                    },
                )
            }),
        );

        Ok(CCProtocolHandles {
            protocol_tag: CCProtocolTag("terminal"),
            sender: in_tx,
            receiver: out_rx,
        })
    }
}
