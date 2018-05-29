use config::Config;
use std::collections::HashMap;
pub use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
pub use tokio::{prelude::*, runtime::Runtime};
pub use tokio_core::reactor::Timeout;
pub use tokio_threadpool::{blocking, BlockingError};

#[cfg(feature = "discord_protocol")]
pub mod discord;
#[cfg(feature = "irc_protocol")]
pub mod irc;
#[cfg(feature = "terminal_protocol")]
pub mod terminal;

pub type CCResult<T> = Result<T, String>;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CCAuthorTag(&'static str);
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CCChannelTag(&'static str);
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CCProtocolTag(&'static str);

#[derive(Clone, Debug)]
pub enum CCMessage {
    Message {
        author: CCAuthorTag,
        source_channel: CCChannelTag,
        raw_contents: String,
        contents: Vec<CCMessageFragment>,
    },
    Control(CCControl),
}

#[derive(Clone, Debug)]
pub enum CCMessageFragment {
    Command(CCCommand),
    Plain(String),
}

#[derive(Clone, Debug)]
pub enum CCCommand {
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum CCControl {
    Shutdown,
}

pub struct CCProtocolHandles {
    pub protocol_tag: CCProtocolTag,
    pub sender: Sender<CCMessage>,
    pub receiver: Receiver<CCMessage>,
}

pub trait CCProtocol {
    fn initialize(runtime: &mut Runtime) -> CCResult<CCProtocolHandles>;
}

pub mod config {
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Protocol {
        protocol_tag: String,
        sources: Vec<Source>,
        destinations: Vec<Destination>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Source {}

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Destination {}
}

#[derive(Debug)]
pub struct ConcordCore {
    //ch_map_ref: Arc<Vec<HashMap<CCProtocolTag, Vec<CCChannelTag>>>>,
    config: Config,
    runtime: Runtime,
    control_sender: Sender<CCControl>,
    control_receiver: Receiver<CCControl>,
    protocol_senders: Arc<RwLock<HashMap<CCProtocolTag, Sender<CCMessage>>>>,
}

impl ConcordCore {
    pub fn new(config: Config) -> CCResult<ConcordCore> {
        let runtime = Runtime::new()
            .map_err(|e| -> String { format!("error creating tokio runtime: {}", e) })?;
        let (command_sender, command_receiver) = channel();
        let protocol_senders = Arc::new(RwLock::new(HashMap::new()));
        /*let raw_protocols = config
            .get::<Vec<config::Protocol>>("protocol")
            .map_err(|e| -> String { format!("config error: {}", e) })?;*/
        Ok(ConcordCore {
            config,
            runtime,
            control_sender: command_sender,
            control_receiver: command_receiver,
            protocol_senders,
        })
    }

    /*fn map_channel(
        ch_map_ref: &Arc<Vec<HashMap<CCProtocolTag, Vec<CCChannelTag>>>>,
        source_protocol: CCProtocolTag,
        source_channel: &CCChannelTag,
    ) -> Vec<(CCProtocolTag, CCChannelTag)> {
        debug!(
            "+> Mapping channels for {}-{}",
            source_protocol, &source_channel
        );
        let mut mapped = Vec::<(CCProtocolTag, CCChannelTag)>::new();
        for p_ch_map in ch_map_ref.iter() {
            let mut should_map = false;
            'outer: for (protocol, ch_vec) in p_ch_map {
                if protocol == &source_protocol {
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
                        if !(protocol == &source_protocol && channel == source_channel) {
                            mapped.push((protocol.clone(), channel.clone()));
                        }
                    }
                }
            }
        }
        debug!("+> Recipients: {:?}", &mapped);
        mapped
    }*/

    pub fn initialize_protocol<T>(&mut self, _protocol: T) -> CCResult<&mut Self>
    where
        T: CCProtocol,
    {
        let CCProtocolHandles {
            protocol_tag,
            sender,
            receiver,
        } = <T>::initialize(&mut self.runtime)?;
        self.protocol_senders
            .write()
            .unwrap()
            .insert(protocol_tag, sender);

        let control_sender = self.control_sender.clone();
        self.runtime
            .spawn(stream::iter_ok(receiver).for_each(move |msg| {
                trace!("Received message: {:?}", msg);
                match msg {
                    CCMessage::Message { contents, .. } => for fragment in &contents {
                        if let CCMessageFragment::Command(cmd) = fragment {
                            match cmd {
                                Shutdown => control_sender.send(CCControl::Shutdown).unwrap(),
                                _ => warn!("unhandled command {:?}!", cmd),
                            }
                        }
                    },
                    _ => (),
                }
                Ok(())
            }));

        self.runtime.spawn(future::poll_fn(|| {
            blocking(|| loop {
                trace!("tick");
                thread::sleep(Duration::new(1, 0));
            }).map_err(|_| panic!("the threadpool shut down"))
        }));

        /*let CCProtocolHandles {
            protocol_tag,
            sender,
            receiver,
            join_handle,
        } = result.unwrap();
        {
            let mut locked = self.p_map_ref.write().unwrap();
            locked.insert(protocol_tag, sender);
        }
        let p_map_ref_clone = self.p_map_ref.clone();
        let ch_map_ref_clone = self.ch_map_ref.clone();
        self.p_handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            for message in receiver.wait() {
                if let Ok(msg) = message {
                    let p_map = p_map_ref_clone.read().unwrap();
                    for (protocol, channel) in
                        ConcordCore::map_channel(&ch_map_ref_clone, protocol_tag, &msg.channel)
                    {
                        if let Some(p_in) = p_map.get(&protocol) {
                            debug!(
                                "Relaying from {}-{} to {}-{}: {:?}",
                                protocol_tag, &msg.channel, &protocol, &channel, &msg,
                            );
                            let mut t_msg = msg.clone();
                            t_msg.channel = channel;
                            if let Err(e) = p_in.clone().send(t_msg).wait() {
                                error!(
                                    "Linker failed to transmit from {} to {}: {}",
                                    protocol_tag, &protocol, e
                                );
                            }
                        }
                    }
                }
            }
            join_handle.join().unwrap();
        }));*/
        Ok(self)
    }

    pub fn spawn_future<F>(&mut self, future: F) -> &mut Self
    where
        F: Future<Item = (), Error = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
        self
    }

    pub fn run(self) {
        let runtime_future = self.runtime.shutdown_on_idle();

        let protocol_senders = self.protocol_senders.clone();
        stream::iter_ok::<_, ()>(self.control_receiver)
            .for_each(|cmd| {
                info!("Control: {:?}", cmd);
                match cmd {
                    Shutdown => {
                        let msg = CCMessage::Control(CCControl::Shutdown);
                        for sender in protocol_senders.read().unwrap().values() {
                            sender.send(msg.clone()).unwrap();
                        }
                    }
                    _ => warn!("unhandled command {:?}!", cmd),
                }
                Ok(())
            })
            .wait()
            .unwrap();

        runtime_future.wait().unwrap();
    }
}

#[cfg(test)]
mod test {
    use config;
    use protocol::terminal::Terminal;
    use protocol::*;

    #[test]
    fn config() {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();

        let mut p_linker = ConcordCore::new(&config);
        debug!("Linker dump: {:?}", p_linker);
    }

    #[test]
    fn basic_relaying() {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();

        let mut p_linker = ConcordCore::new(&config);
        p_linker
            .spawn_relay_thread(Terminal::new(&config))
            .join_all();
    }
}
