use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::{input, KeyEvent, InputEvent};

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub exit_key: KeyEvent,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: KeyEvent::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();

        let input_handle = {
            let tx = tx.clone();

            thread::spawn(move || {
                let stdin = input();
				let reader = stdin.read_sync();

                for event in reader {
					match event {
						InputEvent::Keyboard(key) => {
							if let Err(_) = tx.send(Event::Input(key.clone())) {
								return;
							}

							if key == KeyEvent::Char('q') {
								return;
							}
						}
						_ => {}
					}
                }
            })
        };

        let tick_handle = {
            let tx = tx.clone();

            thread::spawn(move || {
                let tx = tx.clone();

                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(config.tick_rate);
                }
            })
        };

        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}