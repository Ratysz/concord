extern crate irc as irc_crate;
extern crate serenity;
extern crate text_io;

pub use config::Config as OmniConfig;
pub use futures::{Future, Sink, Stream};
pub use futures::sync::mpsc::{channel, Receiver, Sender};
pub use std::thread;
pub use std::thread::JoinHandle;

pub use ProtocolTag;
pub use ChannelTag;
pub use OmniMessage;
pub use OmniProtocol;
pub use OmniProtocolResult;

pub mod irc;
pub mod discord;
pub mod terminal;
