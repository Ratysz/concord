extern crate config;
extern crate futures;
#[cfg(feature = "irc_feature")]
extern crate irc as irc_crate;
#[macro_use]
extern crate log;
#[cfg(feature = "discord_feature")]
extern crate serenity;
#[cfg(feature = "terminal_feature")]
#[macro_use]
extern crate text_io;

mod protocol;
pub use protocol::{OmniProtocol, ProtocolLinker};

#[cfg(feature = "discord_feature")]
pub use protocol::discord::Discord;

#[cfg(feature = "irc_feature")]
pub use protocol::irc::Irc;

#[cfg(feature = "terminal_feature")]
pub use protocol::terminal::Terminal;
