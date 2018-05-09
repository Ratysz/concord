use protocol::*;

impl From<String> for CCMessage {
    fn from(text: String) -> Self {
        let mut vec: Vec<&str> = text.splitn(2, "|").collect();
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
        }
    }
}

pub struct Terminal;

impl CCProtocol for Terminal {
    fn new(config: &OmniConfig) -> CCProtocolInitResult {
        trace!("Starting up.");
        let (in_tx, in_rx) = channel::<CCMessage>(100);
        let (out_tx, out_rx) = channel::<CCMessage>(100);

        trace!("Configured, spawning threads.");
        let handle = thread::spawn(move || {
            trace!("Sender thread spawned.");
            let handle = thread::spawn(move || {
                trace!("Receiver thread spawned.");
                for message in in_rx.wait() {
                    trace!("Received message: {:?}", &message);
                    if let Ok(msg) = message {
                        println!("{}|{}", msg.channel, msg.text);
                    }
                }
                trace!("Receiver thread done.");
            });

            loop {
                let line: String = read!("{}\n");
                trace!("Sending message: {:?}", &line);
                if let Err(e) = out_tx.clone().send(CCMessage::from(line)).wait() {
                    error!("Terminal failed to transmit: {}", e);
                }
            }
            trace!("Sender thread done, joining.");

            handle.join().unwrap();
            trace!("Threads joined.");
        });
        trace!("Threads spawned.");

        Ok(CCProtocolInitOk {
            protocol_tag: "terminal",
            sender: in_tx,
            receiver: out_rx,
            join_handle: handle,
        })
    }
}
