use protocol::*;

impl From<String> for CCMessage {
    fn from(text: String) -> Self {
        CCMessage {
            author: CCAuthorTag("term"),
            source_channel: CCChannelTag("term"),
            raw_contents: text,
            contents: Vec::new(),
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
        let (in_tx, in_rx) = channel::<CCMessage>(100);
        let (out_tx, out_rx) = channel::<CCMessage>(100);

        runtime.spawn(in_rx.for_each(|msg| {
            trace!("Received message: {:?}", msg);
            println!("{:#?}", msg);
            Ok(())
        }));

        runtime.spawn(future::poll_fn(move || {
            blocking(|| loop {
                let line: String = read!("{}\n");
                trace!("Sending message: {:?}", &line);
                if let Err(e) = out_tx.clone().send(CCMessage::from(line)).wait() {
                    error!("Terminal failed to transmit: {}", e);
                }
            }).map_err(|_| panic!("the threadpool shut down"))
        }));

        Ok(CCProtocolHandles {
            protocol_tag: "terminal",
            sender: in_tx,
            receiver: out_rx,
        })
    }
}
