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

use protocol::discord;
use protocol::irc;

pub type ProtocolTag = &'static str;
pub type ChannelTag = &'static str;

#[derive(Clone, Debug)]
pub struct OmniMessage {
    channel: ChannelTag,
    text: String,
}

pub type OmniProtocolResult = Result<
    (
        ProtocolTag,
        Sender<OmniMessage>,
        Receiver<OmniMessage>,
        JoinHandle<()>,
    ),
    &'static str,
>;

pub trait OmniProtocol {
    fn new(config: &config::Config) -> OmniProtocolResult;
}

struct ProtocolLinker {
    p_map_ref: Arc<Mutex<HashMap<ProtocolTag, Sender<OmniMessage>>>>,
    p_handles: Vec<JoinHandle<()>>,
    ch_map_ref: Arc<Vec<HashMap<ProtocolTag, ChannelTag>>>,
}

impl ProtocolLinker {
    fn new(config: &config::Config) -> Self {
        let mut ch_map = Vec::new();
        ch_map.push({
            let mut map = HashMap::new();
            map.insert("discord", "409314585137512450");
            map.insert("irc", "#ratys-bot-test");
            map
        });
        let ch_map_ref = Arc::new(ch_map);
        ProtocolLinker {
            p_map_ref: Arc::new(Mutex::new(HashMap::new())),
            p_handles: Vec::new(),
            ch_map_ref,
        }
    }

    fn map_channel(
        ch_map_ref: &Arc<Vec<HashMap<ProtocolTag, ChannelTag>>>,
        source_channel: &ChannelTag,
        dest_protocol: &ProtocolTag,
    ) -> Vec<ChannelTag> {
        let mut ch_list = Vec::<ChannelTag>::new();
        for map in ch_map_ref.iter() {
            for s_channel in map.values().collect::<Vec<&ChannelTag>>() {
                if s_channel == source_channel {
                    for (protocol, d_channel) in map {
                        if protocol == dest_protocol {
                            ch_list.push(d_channel);
                        }
                    }
                }
            }
        }
        ch_list
    }

    fn spawn_relay_thread(&mut self, result: OmniProtocolResult) -> &mut Self {
        let (p_str, p_in, p_out, p_handle) = result.unwrap();
        {
            let mut locked = self.p_map_ref.lock().unwrap();
            locked.insert(p_str, p_in);
        }
        let p_map_ref_clone = self.p_map_ref.clone();
        let ch_map_ref_clone = self.ch_map_ref.clone();
        self.p_handles.push(thread::spawn(move || {
            for message in p_out.wait() {
                if let Ok(mut msg) = message {
                    let p_map = { p_map_ref_clone.lock().unwrap().clone() };
                    for (protocol, p_input) in p_map {
                        for ch in
                            ProtocolLinker::map_channel(&ch_map_ref_clone, &msg.channel, &protocol)
                        {
                            if !(protocol == p_str && msg.channel == ch) {
                                info!(
                                    "[LINKER] Relaying from {} to {}: {:?}",
                                    p_str,
                                    protocol,
                                    msg.clone()
                                );
                                msg.channel = ch;
                                if let Err(e) = p_input.clone().send(msg.clone()).wait() {
                                    error!("[LINKER] Failed to relay: {}", e);
                                }
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

    let mut p_linker = ProtocolLinker::new(&config);
    p_linker
        .spawn_relay_thread(discord::Discord::new(&config))
        .spawn_relay_thread(irc::Irc::new(&config))
        .join_all();
}
