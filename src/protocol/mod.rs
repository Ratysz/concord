use config::Config;
pub use failure::Fail;
pub use futures::sink::Sink;
pub use futures::stream::Stream;
pub use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use std::collections::HashMap;
//pub use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
pub use tokio::prelude::*;
pub use tokio::runtime::current_thread::Runtime;
pub use tokio::spawn;
pub use tokio_threadpool::{blocking, BlockingError};

#[cfg(feature = "discord_protocol")]
pub mod discord;
#[cfg(feature = "irc_protocol")]
pub mod irc;
#[cfg(feature = "terminal_protocol")]
pub mod terminal;

pub type CCResult<T> = Result<T, String>;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AuthorTag(pub &'static str);
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChannelTag(pub &'static str);
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProtocolTag(pub &'static str);

#[derive(Clone, Debug)]
pub enum Message {
    Message {
        author: AuthorTag,
        source: ChannelTag,
        raw_contents: String,
        contents: Vec<MessageFragment>,
    },
    Control(Command),
}

#[derive(Clone, Debug)]
pub enum MessageFragment {
    Formatting,
    Plain(String),
}

#[derive(Clone, Debug)]
pub enum Command {
    Shutdown,
}

pub struct ProtocolHandles {
    pub protocol_tag: ProtocolTag,
    pub sender: UnboundedSender<Message>,
    pub receiver: UnboundedReceiver<Message>,
}

pub trait Protocol {
    fn initialize(self, runtime: &mut Runtime) -> CCResult<ProtocolHandles>;
}

pub mod config {
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Protocol {
        source: Vec<Source>,
        destination: Vec<Destination>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Source {}

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Destination {}
}

#[derive(Debug)]
pub struct ConcordCore {
    config: Config,
    runtime: Runtime,
    command_sender: UnboundedSender<Command>,
    command_receiver: UnboundedReceiver<Command>,
    protocol_senders: Arc<RwLock<HashMap<ProtocolTag, UnboundedSender<Message>>>>,
}

impl ConcordCore {
    pub fn new(config: &Config) -> CCResult<ConcordCore> {
        let runtime = Runtime::new()
            .map_err(|e| -> String { format!("error creating tokio runtime: {}", e) })?;
        let (command_sender, command_receiver) = unbounded();
        let protocol_senders = Arc::new(RwLock::new(HashMap::new()));
        /*let raw_protocols = config
            .get::<Vec<config::Protocol>>("protocol")
            .map_err(|e| -> String { format!("config error: {}", e) })?;*/
        Ok(ConcordCore {
            config: config.clone(),
            runtime,
            command_sender,
            command_receiver,
            protocol_senders,
        })
    }

    pub fn initialize_protocol<T>(&mut self, protocol: T) -> CCResult<&mut Self>
    where
        T: Protocol,
    {
        let ProtocolHandles {
            protocol_tag,
            sender,
            receiver,
        } = protocol.initialize(&mut self.runtime)?;

        let protocol_config = self.config.get::<config::Protocol>(protocol_tag.0).unwrap();
        debug!("{} config: {:?}", protocol_tag.0, protocol_config);

        self.protocol_senders
            .write()
            .unwrap()
            .insert(protocol_tag, sender);

        let control_sender = self.command_sender.clone();
        self.runtime.spawn(receiver.for_each(move |message| {
            trace!("Received message: {:?}", message);
            match message {
                Message::Message { contents, .. } => for fragment in &contents {
                    match fragment {
                        MessageFragment::Plain(text) => if text.contains("shutdown") {
                            trace!("Sending control: {:?}", &Command::Shutdown);
                            spawn(
                                control_sender
                                    .clone()
                                    .send(Command::Shutdown)
                                    .then(|_| Ok(())),
                            );
                        },

                        _ => unimplemented!(),
                    }
                },
                Message::Control(command) => {
                    trace!("Sending control: {:?}", &command);
                    spawn(control_sender.clone().send(command).then(|_| Ok(())));
                }
            }
            Ok(())
        }));
        Ok(self)
    }

    /*pub fn spawn_future<F>(&mut self, future: F) -> &mut Self
    where
        F: Future<Item = (), Error = ()> + Send + 'static,
    {
        self.runtime.spawn(future);
        self
    }*/

    pub fn run(mut self) {
        let command_rx = self.command_receiver;
        let protocol_txs = {
            let mut vec = Vec::new();
            for sender in self.protocol_senders.read().unwrap().values() {
                vec.push(sender.clone());
            }
            vec
        };

        self.runtime.spawn(command_rx.for_each(move |command| {
            info!("Control: {:?}", command);
            match command {
                Command::Shutdown => {
                    let message = Message::Control(Command::Shutdown);
                    for sender in &protocol_txs {
                        trace!("Relaying control {:?} to {:?}", &Command::Shutdown, &sender);
                        spawn(sender.clone().send(message.clone()).then(|_| Ok(())));
                    }
                    debug!("Terminating.");
                    return Err(());
                }
            }
        }));

        self.runtime.run().unwrap();

        info!("Clean shutdown!");
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
