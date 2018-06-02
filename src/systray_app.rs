use concord_core::protocol::*;
use std::time;
use systray;
use tokio;
use tokio_timer;

pub struct SysTray;

use futures;
use std;

pub struct ReceiverStream<T>(std::sync::mpsc::Receiver<T>);

impl<T> futures::stream::Stream for ReceiverStream<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        match self.0.try_recv() {
            Ok(message) => Ok(futures::Async::Ready(Some(message))),
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => Ok(futures::Async::NotReady),
                std::sync::mpsc::TryRecvError::Disconnected => Ok(futures::Async::Ready(None)),
            },
        }
    }
}

fn periodic_wake_up(
    task: futures::task::Task,
    period: std::time::Duration,
) -> futures::sync::oneshot::Sender<()> {
    let (kill_tx, kill_rx) = futures::sync::oneshot::channel::<()>();
    tokio::spawn(future::loop_fn(
        (period, task, kill_rx),
        |(period, task, mut kill_rx)| {
            let should_continue = Ok(futures::Async::NotReady) == kill_rx.poll();
            tokio_timer::Delay::new(std::time::Instant::now() + period)
                .map_err(|_| ())
                .and_then(move |_| {
                    task.notify();
                    if should_continue {
                        return Ok(future::Loop::Continue((period, task, kill_rx)));
                    }
                    Ok(future::Loop::Break(()))
                })
        },
    ));
    kill_tx
}

impl Protocol for SysTray {
    fn initialize(self, runtime: &mut Runtime) -> CCResult<ProtocolHandles> {
        debug!("Initializing.");
        let (in_tx, in_rx) = unbounded();
        let (out_tx, out_rx) = unbounded();
        let (tray_tx, tray_rx) = std::sync::mpsc::channel();
        let tray = {
            let tray_tx = tray_tx.clone();
            systray::api::api::Window::new(tray_tx).unwrap()
        };
        tray.add_menu_entry(0, &"quit".to_string()).unwrap();

        runtime.spawn(future::lazy(|| {
            let task = futures::task::current();
            let kill_tx = periodic_wake_up(task, time::Duration::from_millis(16));
            ReceiverStream(tray_rx)
                .for_each(move |message| {
                    trace!("Tray received message: {}", message.menu_index);
                    match message.menu_index {
                        0 => {
                            debug!("Tray sending: {:?}", &Command::Shutdown);
                            tokio::spawn(
                                out_tx
                                    .clone()
                                    .send(Message::Control(Command::Shutdown))
                                    .then(|_| Ok(())),
                            );
                        }
                        1 => {
                            debug!("Tray terminating.");
                            tray.shutdown().unwrap();
                            return Err(());
                        }
                        _ => unimplemented!(),
                    }
                    Ok(())
                })
                .then(move |_| {
                    kill_tx.send(()).unwrap();
                    Ok(())
                })
        }));

        runtime.spawn(in_rx.for_each(move |message| {
            trace!("Received message: {:?}", message);
            match message {
                Message::Control(command) => match command {
                    Command::Shutdown => {
                        if let Err(e) = tray_tx.send(systray::SystrayEvent { menu_index: 1 }) {
                            error!("Failed to transmit: {}", e);
                        }
                        debug!("Terminating.");
                        return Err(());
                    }
                },
                _ => unimplemented!(),
            }
            Ok(())
        }));

        Ok(ProtocolHandles {
            protocol_tag: ProtocolTag("systray"),
            sender: in_tx,
            receiver: out_rx,
        })
    }
}
