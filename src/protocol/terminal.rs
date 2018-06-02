use protocol::*;
use std::thread;

impl From<String> for Message {
    fn from(text: String) -> Self {
        let contents = vec![MessageFragment::Plain(text.clone())];
        Message::Message {
            author: AuthorTag("term"),
            source: ChannelTag("term"),
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

impl Protocol for Terminal {
    fn initialize(self, runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        debug!("Initializing.");
        let (in_tx, in_rx) = unbounded();
        let (out_tx, out_rx) = unbounded();

        {
            let in_tx = in_tx.clone();
            thread::spawn(move || loop {
                let line: String = read!("{}\n");
                trace!("Fat thread sending line: {:?}", &line);
                if let Err(e) = in_tx.clone().send(Message::from(line)).wait() {
                    error!("Fat thread failed to transmit: {}", e);
                }
            });
        }

        runtime.spawn(in_rx.for_each(move |message| {
            trace!("Received message: {:?}", &message);
            let mut send = false;
            match message {
                Message::Control(command) => match command {
                    Command::Shutdown => {
                        debug!("Terminating.");
                        return Err(());
                    }
                },
                Message::Message {
                    ref author,
                    ref raw_contents,
                    ..
                } => if author == &AuthorTag("term") {
                    send = true;
                } else {
                    trace!("Posting message: {:?}", &message);
                    println!("{:?}", raw_contents);
                },
            }
            if send {
                trace!("Sending message: {:?}", &message);
                spawn(out_tx.clone().send(message).then(|_| Ok(())));
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
