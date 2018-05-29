#![windows_subsystem = "windows"]
extern crate concord_core;
extern crate config;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate systray;
extern crate tokio;
extern crate tokio_threadpool;

use concord_core::protocol::*;
use concord_core::*;
use simplelog::TermLogger;
use std::time;

struct SysTray;

impl CCProtocol for SysTray {
    fn initialize(runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        let (in_tx, in_rx) = channel::<CCMessage>();
        let (out_tx, out_rx) = channel::<CCMessage>();
        let (intra_tx, intra_rx) = channel::<()>();

        runtime.spawn(future::loop_fn((in_rx, intra_tx), |(in_rx, intra_tx)| {
            if let Ok(message) = in_rx.recv_timeout(time::Duration::from_secs(1)) {
                if let CCMessage::Control(command) = message {
                    match command {
                        Command::Shutdown => {
                            intra_tx.send(()).unwrap();
                            trace!("Receiver task done.");
                            return Ok(future::Loop::Break(()));
                        }
                    }
                }
            }
            Ok(future::Loop::Continue((in_rx, intra_tx)))
        }));

        runtime.spawn(
            future::lazy(|| {
                let (tray_tx, tray_rx) = channel();
                let tray = systray::api::api::Window::new(tray_tx).unwrap();
                tray.add_menu_entry(0, &"quit".to_string()).unwrap();
                Ok((tray_rx, tray))
            }).and_then(|(tray_rx, tray)| {
                future::loop_fn(
                    (out_tx, intra_rx, tray_rx, tray),
                    |(out_tx, intra_rx, tray_rx, tray)| {
                        if intra_rx.try_recv().is_err() {
                            if let Ok(_) = tray_rx.recv_timeout(time::Duration::from_secs(1)) {
                                trace!("SysTray sending shutdown command.");
                                out_tx.send(CCMessage::Control(Command::Shutdown)).unwrap();
                            }
                            Ok(future::Loop::Continue((out_tx, intra_rx, tray_rx, tray)))
                        } else {
                            trace!("Sender task done.");
                            tray.shutdown().unwrap();
                            Ok(future::Loop::Break(()))
                        }
                    },
                )
            }),
        );

        Ok(ProtocolHandles {
            protocol_tag: ProtocolTag("systray"),
            sender: in_tx,
            receiver: out_rx,
        })
    }
}

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
