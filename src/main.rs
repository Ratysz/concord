#![windows_subsystem = "windows"]
extern crate chrono;
extern crate concord_core;
extern crate config;
extern crate fern;
#[macro_use]
extern crate log;
extern crate systray;
extern crate tokio;
extern crate tokio_threadpool;

use concord_core::*;

mod systray_app;

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{:<5}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level().to_string(),
                record.target(),
                message
            ))
        })
        .level_for("tokio_threadpool", log::LevelFilter::Debug)
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let config = {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();
        config
    };

    let mut concord = ConcordCore::new(config).expect("Could not initialize Concord!");
    concord
        .initialize_protocol(systray_app::SysTray)
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
