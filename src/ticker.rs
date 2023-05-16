use std::{sync::mpsc::Sender, thread::{self, JoinHandle}};
use timer::{MessageTimer, Guard};

pub struct Ticker {
    mtimer: MessageTimer<()>,
    ticker: Guard,
    handle: JoinHandle<()>
}

impl Ticker {
    pub fn new<F>(tx: Sender<()>, f: F) -> Self where F: 'static + Fn() -> () + Send {
        let mtimer = timer::MessageTimer::new(tx);
        let ticker = mtimer.schedule_repeating(chrono::Duration::microseconds(16667), ());
        let handle = thread::spawn(f);

        Self {
            mtimer,
            ticker,
            handle
        }
    }
}