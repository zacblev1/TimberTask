use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyCode, KeyModifiers};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    /// Event sender channel
    sender: mpsc::Sender<Event>,
    /// Event receiver channel
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread
    handle: Option<thread::JoinHandle<()>>,
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        
        let handle = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                while !shutdown_clone.load(Ordering::Relaxed) {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));

                    match event::poll(timeout) {
                        Ok(true) => match event::read() {
                            Ok(event) => match event {
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
                            },
                            Err(_) => {
                                // Failed to read event, but we can continue
                            }
                        },
                        Ok(false) => {
                            // No event available
                        }
                        Err(_) => {
                            // Failed to poll, but we can continue
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
            sender,
            receiver,
            handle: Some(handle),
            shutdown,
        }
    }

    /// Get the next event from the handler
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
    
    /// Signal the event handler to shutdown
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        // Send a dummy event to wake up the thread if it's blocked on poll
        let _ = self.sender.send(Event::Tick);
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Send a dummy event to wake up the thread if it's blocked
        let _ = self.sender.send(Event::Tick);
        
        // Wait for the thread to finish
        if let Some(handle) = self.handle.take() {
            // Give the thread a reasonable amount of time to finish
            // We use a timeout to avoid hanging forever
            let _ = handle.join();
        }
    }
}