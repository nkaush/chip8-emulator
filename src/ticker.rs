use std::{sync::mpsc::channel, thread::{self, JoinHandle}};
use timer::{MessageTimer, Guard};

pub struct Ticker {
    _mtimer: MessageTimer<()>,
    _ticker: Guard,
    _handle: JoinHandle<()>
}

impl Ticker {
    pub fn new<F>(f: F) -> Self where F: 'static + Send + Fn() {
        let (tx, rx) = channel();
        let _mtimer = timer::MessageTimer::new(tx);
        let _ticker = _mtimer.schedule_repeating(chrono::Duration::microseconds(16667), ());
        let _handle = thread::spawn(move || {
            rx.iter().for_each(|_| f())
        });

        Self {
            _mtimer,
            _ticker,
            _handle
        }
    }
}