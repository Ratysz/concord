#![windows_subsystem = "windows"]
extern crate concord_core;
extern crate config;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate systray;
extern crate tokio;
extern crate tokio_threadpool;

use concord_core::*;
use simplelog::TermLogger;

mod systray_app;
use systray_app::SysTray;

fn main() {
    TermLogger::init(simplelog::LevelFilter::Trace, simplelog::Config::default()).unwrap();
    let config = {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();
        config
    };

    let mut concord = ConcordCore::new(config).expect("Could not initialize Concord!");
    concord
        .initialize_protocol(SysTray)
        .expect("Couldn't initialize SysTray!");

    /*#[cfg(feature = "discord_protocol")]
    concord
        //.initialize_protocol(Discord::new(&config))
        .initialize_protocol(Discord)
        .expect("Could not initialize Discord!");

    #[cfg(feature = "irc_protocol")]
    concord
        //.initialize_protocol(Irc::new(&config))
        .initialize_protocol(Irc)
        .expect("Could not initialize Irc!");*/

    #[cfg(feature = "terminal_protocol")]
    concord
        //.initialize_protocol(Terminal::new(&config))
        .initialize_protocol(Terminal)
        .expect("Could not initialize Terminal!");

    concord.run();
}
