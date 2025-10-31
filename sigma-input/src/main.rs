use chrono::Local;
use evdev::*;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

/// Append a line to a user-local log (no root required for writing the log).
fn log_line(log_file: &PathBuf, line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_file) {
        let now = Local::now();
        let _ = writeln!(f, "{}  {}", now.format("%Y-%m-%d %H:%M:%S"), line);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Expect device path as first arg
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} /dev/input/eventN <log file>", args[0]);
        eprintln!("Find the correct device with `sudo evtest` or `cat /proc/bus/input/devices`.");
        process::exit(1);
    }
    let dev_path = &args[1];
    let log_file_str = &args[2];
    let log_file = PathBuf::from(log_file_str);

    if !Path::new(dev_path).exists() {
        eprintln!("Device {} does not exist.", dev_path);
        process::exit(1);
    }

    // Open the device (requires appropriate permissions / root)
    let mut device = Device::open(dev_path).map_err(|e| {
        format!(
            "Failed to open {}: {}. Are you root or have device access?",
            dev_path, e
        )
    })?;

    log_line(
        &log_file,
        &format!(
            "Opened device: {} ({})",
            dev_path,
            device.name().unwrap_or("unknown")
        ),
    );

    let mut alt_down = false;

    loop {
        for ev in device.fetch_events()? {
            if let EventSummary::Key(_key_event, key_code, value) = ev.destructure() {
                match key_code {
                    KeyCode::KEY_LEFTALT => {
                        log_line(
                            &log_file,
                            &format!("LEFTALT: key code: {key_code:?}, value: {value}"),
                        );
                        if value == 1 {
                            alt_down = true;
                        } else if value == 0 {
                            alt_down = false;
                        }
                    }
                    key_code => {
                        if alt_down {
                            log_line(
                                &log_file,
                                &format!("key {key_code:?} got with value {value}"),
                            );
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
