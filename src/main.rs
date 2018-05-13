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
use tokio::prelude::*;
use tokio_threadpool::blocking;

fn main() {
    TermLogger::init(simplelog::LevelFilter::Trace, simplelog::Config::default()).unwrap();
    let config = {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();
        config
    };

    let mut concord = ConcordCore::new(config).expect("Could not initialize Concord!");

    concord.spawn_future(future::poll_fn(|| {
        blocking(|| {
            match systray::Application::new() {
                Ok(mut app) => {
                    //app.set_icon_from_resource(&"/usr/share/gxkb/flags/ua.png".to_string()).ok();
                    app.add_menu_item(&"Quit".to_string(), |window| {
                        info!("SysTray stopping.");
                        window.quit();
                    }).ok();
                    info!("SysTray initialized.");
                    app.wait_for_message();
                }
                Err(e) => {
                    error!("Couldn't create systray app: {}", e);
                }
            }
        }).map_err(|_| panic!("the threadpool shut down"))
    }));

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
