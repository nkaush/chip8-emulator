use chip8::cpu::Cpu;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut cpu = Cpu::new(args[1].clone().into())?;

    let (tx, rx) = std::sync::mpsc::channel();
    let timer = timer::MessageTimer::new(tx);

    // Start repeating.
    let _guard = timer.schedule_repeating(chrono::Duration::microseconds(1000), ());

    loop {
        let fetched = cpu.fetch()
            .map_err(|e| {
                eprintln!("{}", cpu);
                eprintln!("{}", cpu.memory);
                format!("{e:?}")
            })?;
        let decoded = cpu.decode(fetched)
            .map_err(|e| {
                eprintln!("{}", cpu);
                eprintln!("{}", cpu.memory);
                format!("{e:?}")
            })?;

        eprintln!("{fetched:04x} => {decoded:?}");
        cpu.execute(decoded)
            .map_err(|e| {
                eprintln!("{}", cpu);
                eprintln!("{}", cpu.memory);
                format!("{e:?}")
            })?;

        rx.recv().unwrap()
    }
}