extern crate config;
extern crate futures;
#[cfg(feature = "irc_protocol")]
extern crate irc as irc_crate;
#[macro_use]
extern crate log;
#[cfg(feature = "discord_protocol")]
extern crate serenity;
#[cfg(feature = "terminal_protocol")]
#[macro_use]
extern crate text_io;

mod protocol;
pub use protocol::{CCProtocol, ConcordCore};

#[cfg(feature = "discord_protocol")]
pub use protocol::discord::Discord;

#[cfg(feature = "irc_protocol")]
pub use protocol::irc::Irc;

#[cfg(feature = "terminal_protocol")]
pub use protocol::terminal::Terminal;
