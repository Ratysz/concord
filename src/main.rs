#![windows_subsystem = "windows"]
extern crate config;
extern crate futures;
#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::TermLogger;

mod commands;
mod protocol;

use protocol::*;
use protocol::discord;
use protocol::irc;

#[derive(Debug)]
pub struct OmniChannel {
    discord: protocol::serenity::model::id::ChannelId,
}

#[derive(Debug)]
pub struct OmniMessage {
    channel: OmniChannel,
    text: String,
}

pub type OmniProtocolResult = Result<
    (
        &'static str,
        Sender<OmniMessage>,
        Receiver<OmniMessage>,
        JoinHandle<()>,
    ),
    &'static str,
>;

pub trait OmniProtocol {
    fn new(config: config::Config) -> OmniProtocolResult;
}

fn main() {
    TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();
    let mut config = config::Config::default();
    config.merge(config::File::with_name("config")).unwrap();

    let (dis, dis_in, dis_out, dis_handle) = discord::Discord::new(config.clone()).unwrap();
    let (irc, irc_in, irc_out, irc_handle) = irc::Irc::new(config.clone()).unwrap();

    dis_handle.join();
    irc_handle.join();
}
