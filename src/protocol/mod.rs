extern crate irc as irc_crate;
pub extern crate serenity;

pub use OmniChannel;
pub use OmniMessage;
pub use OmniProtocol;
pub use OmniProtocolResult;
pub use futures::{Future, Sink, Stream};
pub use futures::sync::mpsc::{channel, Receiver, Sender};
pub use config::Config as OmniConfig;
pub use std::thread;
pub use std::thread::JoinHandle;

pub mod irc;
pub mod discord;
