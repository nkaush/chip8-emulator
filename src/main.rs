use chip8::cpu::{Cpu, CpuError};
use std::{env, sync::mpsc};
use chrono::Duration;

fn main() -> Result<(), CpuError> {
    let args = env::args().collect::<Vec<_>>();
    let mut cpu = Cpu::new(args[1].clone().into())?;

    let (tx, rx) = mpsc::channel();
    let timer = timer::MessageTimer::new(tx);

    // Start repeating.
    let _guard = timer.schedule_repeating(Duration::microseconds(1000), ());

    loop {
        let fetched = cpu.fetch()
            .or_else(|e| {
                eprintln!("{}", cpu);
                cpu.dump_core();
                format!("{e:?}");
                Err(e)
            })?;
        let decoded = cpu.decode(fetched)
            .or_else(|e| {
                eprintln!("{}", cpu);
                cpu.dump_core();
                format!("{e:?}");
                Err(e)
            })?;

        eprintln!("{fetched:04x} => {decoded:?}");
        cpu.execute(decoded)
            .or_else(|e| {
                eprintln!("{}", cpu);
                cpu.dump_core();
                format!("{e:?}");
                Err(e)
            })?;

        rx.recv().unwrap()
    }
}