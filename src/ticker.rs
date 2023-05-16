use std::{sync::mpsc::Sender, thread::{self, JoinHandle}};
use timer::{MessageTimer, Guard};

pub struct Ticker {
    _mtimer: MessageTimer<()>,
    _ticker: Guard,
    _handle: JoinHandle<()>
}

impl Ticker {
    pub fn new<F>(tx: Sender<()>, f: F) -> Self where F: 'static + Fn() -> () + Send {
        let _mtimer = timer::MessageTimer::new(tx);
        let _ticker = _mtimer.schedule_repeating(chrono::Duration::microseconds(16667), ());
        let _handle = thread::spawn(f);

        Self {
            _mtimer,
            _ticker,
            _handle
        }
    }
}