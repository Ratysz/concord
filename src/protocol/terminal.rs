use protocol::*;
use std::{thread, time};

impl From<String> for CCMessage {
    fn from(text: String) -> Self {
        let mut contents = Vec::new();
        if text.contains("shutdown") {
            contents.push(CCMessageFragment::Command());
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
    fn initialize(runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        trace!("Starting up.");
        let (in_tx, in_rx) = channel::<CCMessage>();
        let (out_tx, out_rx) = channel::<CCMessage>();

        let in_tx_clone = in_tx.clone();
        thread::spawn(move || loop {
            let line: String = read!("{}\n");
            in_tx_clone.send(CCMessage::from(line)).unwrap();
        });

        runtime.spawn(stream::iter_ok(in_rx).for_each(move |message| {
            let mut send = false;
            match message {
                CCMessage::Control(command) => match command {
                    Command::Shutdown => {
                        trace!("Terminating.");
                        return Err(());
                    }
                },
                CCMessage::Message {
                    ref author,
                    ref raw_contents,
                    ..
                } => if author == &CCAuthorTag("term") {
                    send = true;
                } else {
                    println!("{:?}", raw_contents);
                },
            }
            if send {
                trace!("Sending message: {:?}", &message);
                out_tx.send(message).unwrap();
            }
            Ok(())
        }));

        Ok(ProtocolHandles {
            protocol_tag: ProtocolTag("terminal"),
            sender: in_tx,
            receiver: out_rx,
        })
    }
}
