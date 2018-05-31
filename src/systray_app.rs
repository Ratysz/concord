use concord_core::protocol::*;
use std::time;
use systray;

pub struct SysTray;

impl Protocol for SysTray {
    fn initialize(self, runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        let (in_tx, in_rx) = channel::<Message>();
        let (out_tx, out_rx) = channel::<Message>();
        let (intra_tx, intra_rx) = channel::<()>();

        runtime.spawn(future::loop_fn((in_rx, intra_tx), |(in_rx, intra_tx)| {
            if let Ok(message) = in_rx.recv_timeout(time::Duration::from_secs(1)) {
                if let Message::Control(command) = message {
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
                                out_tx.send(Message::Control(Command::Shutdown)).unwrap();
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
