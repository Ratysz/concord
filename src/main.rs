#![windows_subsystem = "windows"]
extern crate concord_core;
extern crate config;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate systray;

use concord_core::*;
use simplelog::TermLogger;

fn main() {
    TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();
    let config = {
        let mut config = config::Config::default();
        config.merge(config::File::with_name("config")).unwrap();
        config
    };

    match systray::Application::new() {
        Ok(mut app) => {
            //app.set_icon_from_file(&"/usr/share/gxkb/flags/ua.png".to_string()).ok();
            /*app.add_menu_item(&"Print a thing".to_string(), |_| {
                println!("Printing a thing!");
            }).ok();
            app.add_menu_item(&"Add Menu Item".to_string(), |window| {
                window
                    .add_menu_item(&"Interior item".to_string(), |_| {
                        println!("what");
                    })
                    .ok();
                window.add_menu_separator().ok();
            }).ok();
            app.add_menu_separator().ok();*/
            app.add_menu_item(&"Quit".to_string(), |window| {
                window.quit();
            }).ok();
            info!("SysTray initialized.");
            app.wait_for_message();
        }
        Err(e) => error!("[APP] Couldn't create systray app: {}", e),
    }

    let mut p_linker = ProtocolLinker::new(&config);

    #[cfg(feature = "discord_protocol")]
    p_linker.spawn_relay_thread(Discord::new(&config));

    #[cfg(feature = "irc_protocol")]
    p_linker.spawn_relay_thread(Irc::new(&config));

    #[cfg(feature = "terminal_protocol")]
    p_linker.spawn_relay_thread(Terminal::new(&config));

    p_linker.join_all();
}
