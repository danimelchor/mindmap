use std::{fs::File, path::PathBuf, thread};

use anyhow::Result;
use colored::Colorize;
use fs2::FileExt;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};

pub fn acquire_lock(path: &PathBuf) -> Result<()> {
    let file = if !path.exists() {
        File::create(path)?
    } else {
        File::open(path)?
    };
    let res = file.try_lock_exclusive();
    if res.is_err() {
        println!("Another instance is running, waiting for it to finish...");
        file.lock_exclusive()?;
    }

    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    thread::spawn(move || {
        if signals.forever().next().is_some() {
            println!("{}", "Received signal, exiting".red());
            file.unlock().expect("Failed to unlock");
            std::process::exit(0);
        }
    });

    Ok(())
}
