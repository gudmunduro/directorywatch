use std::{collections::HashSet, fs, thread, time::Duration};

use anyhow::Result;
use clap::Parser;
use simplelog::TermLogger;

enum FsEntry {
    Directory(String, Vec<FsEntry>),
    File(String),
}

#[derive(Parser, Debug)]
#[command(about = "Application to montior directory for changes", long_about = None)]
struct Args {
    /// The directory to watch
    #[clap(value_parser)]
    directories: Vec<String>,
}

fn main() {
    init_logger();
    let args = Args::parse();

    log::info!("Scanning directory");
    let dir_maps = args.directories.iter().map(|d| scan_directory(d)).collect::<Vec<FsEntry>>();

    log::info!("Monitoring");
    loop {
        for dir_map in &dir_maps {
            match scan_for_changes(&dir_map) {
                Ok(()) => {},
                Err(e) => {
                    log::error!("Error occurred while scanning directory");
                    log::error!("{e:?}");
                }
            }
        }
        thread::sleep(Duration::from_secs_f32(0.5));
    }
}

fn scan_directory(directory: &str) -> FsEntry {
    let entries = fs::read_dir(directory)
        .unwrap()
        .map(|e| {
            let entry = e.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                scan_directory(&entry.path().to_str().unwrap())
            } else {
                FsEntry::File(entry.path().to_string_lossy().to_string())
            }
        })
        .collect();

    FsEntry::Directory(directory.to_string(), entries)
}

fn scan_for_changes(directory: &FsEntry) -> Result<()> {
    use FsEntry::*;
    match directory {
        Directory(path, entries) => {
            let original_entries = entries
                .iter()
                .map(|e| match e {
                    Directory(p, _) => p.to_owned(),
                    File(p) => p.to_owned(),
                })
                .collect::<HashSet<String>>();
            let current_entries = fs::read_dir(path)?
                .map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string()))
                .filter_map(|e| match e {
                    Some(_) => e,
                    None => {
                        log::error!("Failed to check file in directory {path}");
                        None
                    }
                })
                .collect::<HashSet<String>>();

            for diff_entry in current_entries.difference(&original_entries) {
                handle_diff_entry(diff_entry, fs::metadata(diff_entry)?.is_dir())?;
            }

        }
        File(_) => {}
    }

    Ok(())
}

fn handle_diff_entry(path: &str, is_dir: bool) -> Result<()> {
    log::info!("Unauthorized file detected at {path}");
    if is_dir {
        fs::remove_dir(path)?;
    }
    else {
        fs::remove_file(path)?;
    }
    log::info!("Unauthorized file has been removed");

    Ok(())
}

fn init_logger() {
    TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("Failed to init logger");
}