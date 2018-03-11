#![windows_subsystem = "windows"]
extern crate config;
extern crate futures;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate systray;

use futures::{Future, Sink, Stream};
use futures::sync::mpsc::{Receiver, Sender};
use simplelog::TermLogger;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

mod commands;
mod protocol;

use protocol::serenity::model::id::ChannelId as TEMP;
use protocol::discord;
use protocol::irc;

#[derive(Clone, Debug)]
pub struct OmniChannel {
    discord: TEMP,
    irc: String,
}

#[derive(Clone, Debug)]
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

struct ProtocolLinker {
    p_map_ref: Arc<Mutex<HashMap<&'static str, Sender<OmniMessage>>>>,
    p_handles: Vec<JoinHandle<()>>,
}

impl ProtocolLinker {
    fn new() -> Self {
        ProtocolLinker {
            p_map_ref: Arc::new(Mutex::new(HashMap::new())),
            p_handles: Vec::new(),
        }
    }

    fn spawn_relay_thread(&mut self, args: OmniProtocolResult) -> &mut Self {
        let (p_str, p_in, p_out, p_handle) = args.unwrap();
        {
            let mut locked = self.p_map_ref.lock().unwrap();
            locked.insert(p_str, p_in);
        }
        let p_map_ref_clone = self.p_map_ref.clone();
        self.p_handles.push(thread::spawn(move || {
            for message in p_out.wait() {
                if let Ok(msg) = message {
                    let p_map = { p_map_ref_clone.lock().unwrap().clone() };
                    for (k, v) in p_map {
                        if k != p_str {
                            info!(
                                "[LINKER] Relaying from {} to {}: {:?}",
                                p_str,
                                k,
                                msg.clone()
                            );
                            if let Err(e) = v.send(msg.clone()).wait() {
                                error!("[LINKER] Failed to relay: {}", e);
                            }
                        }
                    }
                }
            }
            p_handle.join().unwrap();
        }));
        self
    }

    fn join_all(&mut self) {
        loop {
            match self.p_handles.pop() {
                Some(handle) => handle.join().unwrap(),
                None => break,
            };
        }
    }
}

fn main() {
    TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();
    let mut config = config::Config::default();
    config.merge(config::File::with_name("config")).unwrap();

    /*match systray::Application::new() {
        Ok(mut app) => {
            //app.set_icon_from_file(&"/usr/share/gxkb/flags/ua.png".to_string()).ok();
            app.add_menu_item(&"Print a thing".to_string(), |_| {
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
            app.add_menu_separator().ok();
            app.add_menu_item(&"Quit".to_string(), |window| {
                window.quit();
            }).ok();
            println!("Waiting on message!");
            app.wait_for_message();
        }
        Err(e) => error!("[APP] Couldn't create systray app: {}", e),
    }*/

    let mut p_linker = ProtocolLinker::new();
    p_linker
        .spawn_relay_thread(discord::Discord::new(config.clone()))
        .spawn_relay_thread(irc::Irc::new(config.clone()))
        .join_all();
}
