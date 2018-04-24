pub use config::Config as OmniConfig;
pub use futures::{Future, Sink, Stream};
pub use futures::sync::mpsc::{channel, Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
pub use std::thread;
pub use std::thread::JoinHandle;
use std::time::Duration;

#[cfg(feature = "discord_protocol")]
pub mod discord;
#[cfg(feature = "irc_protocol")]
pub mod irc;
#[cfg(feature = "terminal_protocol")]
pub mod terminal;

pub type CCProtocolTag = String;
pub type ChannelTag = String;

#[derive(Clone, Debug)]
pub struct CCMessage {
    channel: ChannelTag,
    text: String,
}

pub type CCProtocolInitResult = Result<CCProtocolInitOk, &'static str>;

pub struct CCProtocolInitOk {
    pub protocol_tag: CCProtocolTag,
    pub sender: Sender<CCMessage>,
    pub receiver: Receiver<CCMessage>,
    pub join_handle: JoinHandle<()>,
}

pub trait CCProtocol {
    fn new(config: &OmniConfig) -> CCProtocolInitResult;
}

#[derive(Debug)]
pub struct ProtocolLinker {
    p_map_ref: Arc<RwLock<HashMap<CCProtocolTag, Sender<CCMessage>>>>,
    p_handles: Vec<JoinHandle<()>>,
    ch_map_ref: Arc<Vec<HashMap<CCProtocolTag, Vec<ChannelTag>>>>,
}

impl ProtocolLinker {
    pub fn new(config: &OmniConfig) -> Self {
        let ch_map = config
            .get::<Vec<HashMap<ChannelTag, Vec<ChannelTag>>>>("channel")
            .unwrap();
        ProtocolLinker {
            p_map_ref: Arc::new(RwLock::new(HashMap::new())),
            p_handles: Vec::new(),
            ch_map_ref: Arc::new(ch_map),
        }
    }

    fn map_channel(
        ch_map_ref: &Arc<Vec<HashMap<CCProtocolTag, Vec<ChannelTag>>>>,
        source_protocol: &CCProtocolTag,
        source_channel: &ChannelTag,
    ) -> Vec<(CCProtocolTag, ChannelTag)> {
        debug!(
            "+> Mapping channels for {}-{}",
            &source_protocol, &source_channel
        );
        let mut mapped = Vec::<(CCProtocolTag, ChannelTag)>::new();
        for p_ch_map in ch_map_ref.iter() {
            let mut should_map = false;
            'outer: for (protocol, ch_vec) in p_ch_map {
                if protocol == source_protocol {
                    for channel in ch_vec {
                        if channel == source_channel {
                            should_map = true;
                            debug!("| - adding from {:?}", &p_ch_map);
                            break 'outer;
                        }
                    }
                }
            }
            if should_map {
                for (protocol, ch_vec) in p_ch_map {
                    for channel in ch_vec {
                        if !(protocol == source_protocol && channel == source_channel) {
                            mapped.push((protocol.clone(), channel.clone()));
                        }
                    }
                }
            }
        }
        debug!("+> Recipients: {:?}", &mapped);
        mapped
    }

    pub fn spawn_relay_thread(&mut self, result: CCProtocolInitResult) -> &mut Self {
        let CCProtocolInitOk {
            protocol_tag,
            sender,
            receiver,
            join_handle,
        } = result.unwrap();
        {
            let mut locked = self.p_map_ref.write().unwrap();
            locked.insert(protocol_tag.clone(), sender);
        }
        let p_map_ref_clone = self.p_map_ref.clone();
        let ch_map_ref_clone = self.ch_map_ref.clone();
        self.p_handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            for message in receiver.wait() {
                if let Ok(msg) = message {
                    let p_map = p_map_ref_clone.read().unwrap();
                    for (protocol, channel) in
                        ProtocolLinker::map_channel(&ch_map_ref_clone, &protocol_tag, &msg.channel)
                    {
                        if let Some(p_in) = p_map.get(&protocol) {
                            debug!(
                                "Relaying from {}-{} to {}-{}: {:?}",
                                &protocol_tag, &msg.channel, &protocol, &channel, &msg,
                            );
                            let mut t_msg = msg.clone();
                            t_msg.channel = channel;
                            if let Err(e) = p_in.clone().send(t_msg).wait() {
                                error!(
                                    "Linker failed to transmit from {} to {}: {}",
                                    &protocol_tag, &protocol, e
                                );
                            }
                        }
                    }
                }
            }
            join_handle.join().unwrap();
        }));
        self
    }

    pub fn join_all(&mut self) {
        loop {
            match self.p_handles.pop() {
                Some(handle) => handle.join().unwrap(),
                None => break,
            };
        }
    }
}

#[cfg(test)]
mod test {
    use config;
    use protocol::*;
    use protocol::terminal::Terminal;

    #[test]
    fn config() {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();

        let mut p_linker = ProtocolLinker::new(&config);
        debug!("Linker dump: {:?}", p_linker);
    }

    #[test]
    fn basic_relaying() {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();

        let mut p_linker = ProtocolLinker::new(&config);
        p_linker
            .spawn_relay_thread(Terminal::new(&config))
            .join_all();
    }
}
