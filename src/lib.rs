extern crate config;
extern crate failure;
extern crate futures;
#[cfg(feature = "irc_protocol")]
extern crate irc as irc_crate;
#[macro_use]
extern crate log;
extern crate serde;
#[cfg(feature = "discord_protocol")]
extern crate serenity;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "terminal_protocol")]
#[macro_use]
extern crate text_io;
extern crate tokio;
#[cfg(feature = "terminal_protocol")]
extern crate tokio_fs;
extern crate tokio_threadpool;

pub mod protocol;
pub use protocol::{Command, ConcordCore, Protocol};

#[cfg(feature = "discord_protocol")]
pub use protocol::discord::Discord;

#[cfg(feature = "irc_protocol")]
pub use protocol::irc::Irc;

#[cfg(feature = "terminal_protocol")]
pub use protocol::terminal::Terminal;
