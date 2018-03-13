use protocol::*;

impl From<String> for OmniMessage {
    fn from(text: String) -> Self {
        let mut vec: Vec<&str> = text.splitn(2, "|").collect();
        if let (Some(text), Some(channel)) = (vec.pop(), vec.pop()) {
            let msg = OmniMessage {
                channel: channel.to_string(),
                text: text.to_string(),
            };
            debug!("Formed message: {:?}", msg);
            msg
        } else {
            warn!("Malformed message; proper syntax: channel|message");
            OmniMessage {
                channel: "debug".to_string(),
                text: format!("Malformed message sent via terminal: {}", text).to_string(),
            }
        }
    }
}

pub struct Terminal;

impl OmniProtocol for Terminal {
    fn new(config: &OmniConfig) -> OmniProtocolResult {
        debug!("Starting up.");
        let (in_tx, in_rx) = channel::<OmniMessage>(100);
        let (out_tx, out_rx) = channel::<OmniMessage>(100);

        debug!("Configured, spawning threads.");
        let handle = thread::spawn(move || {
            debug!("Sender thread spawned.");
            let handle = thread::spawn(move || {
                debug!("Receiver thread spawned.");
                for message in in_rx.wait() {
                    debug!("Received message: {:?}", &message);
                    if let Ok(msg) = message {
                        println!("{}|{}", msg.channel, msg.text);
                    }
                }
                debug!("Receiver thread done.");
            });

            loop {
                let line: String = read!("{}\n");
                debug!("Sending message: {:?}", &line);
                if let Err(e) = out_tx.clone().send(OmniMessage::from(line)).wait() {
                    error!("Failed to transmit: {}", e);
                }
            }
            debug!("Sender thread done, joining.");

            handle.join().unwrap();
            debug!("Threads joined.");
        });
        debug!("Threads spawned.");

        Ok(("terminal".to_string(), in_tx, out_rx, handle))
    }
}
