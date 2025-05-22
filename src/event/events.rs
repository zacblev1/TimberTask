use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyCode, KeyModifiers};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Terminal events
pub enum Event {
    /// Terminal tick (occurs at regular intervals)
    Tick,
    /// Input event
    Input(KeyEvent),
    /// Terminal resize
    Resize,
}

/// Event handler
pub struct EventHandler {
    /// Event sender channel (unused in struct but needed for thread)
    _sender: mpsc::Sender<Event>,
    /// Event receiver channel
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread
    _handle: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let _handle = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));

                    if event::poll(timeout).expect("Failed to poll for events") {
                        match event::read().expect("Failed to read event") {
                            CrosstermEvent::Key(key) => {
                                // Handle Ctrl+C through the main event loop
                                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                    // Send a special key event that will be handled by the main app
                                    if sender.send(Event::Input(KeyEvent {
                                        code: KeyCode::Char('q'),
                                        modifiers: KeyModifiers::CONTROL,
                                        kind: key.kind,
                                        state: key.state,
                                    })).is_err() {
                                        return;
                                    }
                                } else {
                                    // For all other keys, send them normally
                                    // Tab key can sometimes be detected differently in different terminals
                                    // We handle it uniformly in the main application code
                                    if sender.send(Event::Input(key)).is_err() {
                                        return;
                                    }
                                }
                            }
                            CrosstermEvent::Resize(_, _) => {
                                if sender.send(Event::Resize).is_err() {
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        if sender.send(Event::Tick).is_err() {
                            return;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            _sender: sender,
            receiver,
            _handle,
        }
    }

    /// Get the next event from the handler
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}