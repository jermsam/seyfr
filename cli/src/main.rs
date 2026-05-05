use clap::{Parser, Subcommand};
use seyfr_core::{Core, ProgressSink};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ── Beautiful Console Progress (ANSI, no ratatui) ───────────────────────────

struct ConsoleProgress {
    start: Instant,
    last_bytes: AtomicU64,
    last_time: AtomicU64, // nanos since epoch for speed calc
}

impl ConsoleProgress {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_bytes: AtomicU64::new(0),
            last_time: AtomicU64::new(now.elapsed().as_nanos() as u64),
        }
    }

    fn draw_bar(pct: f64, width: usize) -> String {
        let filled = (pct / 100.0 * width as f64) as usize;
        let empty = width.saturating_sub(filled);
        format!(
            "\x1b[32m{}\x1b[90m{}\x1b[0m",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }

    fn human(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut idx = 0;
        while size >= 1024.0 && idx < UNITS.len() - 1 {
            size /= 1024.0;
            idx += 1;
        }
        format!("{:.1} {}", size, UNITS[idx])
    }
}

impl ProgressSink for ConsoleProgress {
    fn on_file_start(&self, name: String, current: u64, total: u64) {
        println!(
            "\x1b[36m▶\x1b[0m [{:>3}/{:<3}] \x1b[1m{}\x1b[0m",
            current, total, name
        );
    }

    fn on_file_progress(&self, _name: String, bytes: u64, total: u64) {
        if total > 0 {
            let pct = (bytes as f64 / total as f64 * 100.0).min(100.0);
            let bar = Self::draw_bar(pct, 24);

            // Speed calc
            let now_nanos = self.start.elapsed().as_nanos() as u64;
            let last_nanos = self.last_time.swap(now_nanos, Ordering::SeqCst);
            let last_bytes = self.last_bytes.swap(bytes, Ordering::SeqCst);
            let dt = (now_nanos.saturating_sub(last_nanos)) as f64 / 1e9;
            let speed = if dt > 0.0 {
                (bytes.saturating_sub(last_bytes)) as f64 / dt
            } else {
                0.0
            };

            let speed_str = if speed > 0.0 {
                format!("{} /s", Self::human(speed as u64))
            } else {
                String::new()
            };

            print!(
                "\r\x1b[2K  {} {:>5.1}%  {}  {}/{}",
                bar, pct, speed_str, Self::human(bytes), Self::human(total)
            );
            let _ = std::io::stdout().flush();
        } else {
            // Unknown total — show spinner with bytes
            let elapsed = self.start.elapsed().as_secs();
            let spin = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner = spin[(elapsed as usize) % spin.len()];
            print!(
                "\r\x1b[2K  \x1b[36m{}\x1b[0m downloading... {} (total unknown)",
                spinner, Self::human(bytes)
            );
            let _ = std::io::stdout().flush();
        }
    }

    fn on_file_complete(&self, name: String, current: u64, total: u64) {
        println!(
            "\r\x1b[2K  \x1b[32m✓\x1b[0m [{:>3}/{:<3}] \x1b[1m{}\x1b[0m done",
            current, total, name
        );
    }

    fn on_complete(&self, message: String) {
        println!("\x1b[32m✅\x1b[0m \x1b[1m{}\x1b[0m", message);
    }

    fn on_error(&self, message: String) {
        eprintln!("\x1b[31m❌\x1b[0m {}", message);
    }
}

#[derive(Parser)]
#[command(name = "seyfr")]
#[command(about = "Seyfr — peer-to-peer file transfer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a file or folder (returns a ticket)
    Send {
        /// Path to the file or folder to send
        path: String,
        /// Data directory for the node
        #[arg(short, long, default_value = ".seyfr")]
        data_dir: String,
        /// Keep the node running after generating the ticket
        #[arg(short, long)]
        listen: bool,
    },
    /// Receive files from a ticket
    Receive {
        /// Ticket string from the sender
        ticket: String,
        /// Destination directory
        #[arg(short, long, default_value = ".")]
        dest: String,
        /// Data directory for the node
        #[arg(long, default_value = ".seyfr")]
        data_dir: String,
    },
    /// Show the node ID
    NodeId {
        /// Data directory for the node
        #[arg(short, long, default_value = ".seyfr")]
        data_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::NodeId { data_dir } => {
            let core = Core::new(data_dir).expect("Failed to initialize Seyfr core");
            println!("{}", core.node_id());
        }
        Commands::Send { path, data_dir, listen } => {
            run_send(path, data_dir, listen);
        }
        Commands::Receive { ticket, dest, data_dir } => {
            if let Err(e) = std::fs::create_dir_all(&dest) {
                eprintln!("Failed to create destination directory '{}': {}", dest, e);
                std::process::exit(1);
            }
            run_receive(ticket, dest, data_dir);
        }
    }
}

fn run_send(path: String, data_dir: String, listen: bool) {
    let core = Core::new(data_dir).expect("Failed to initialize Seyfr core");
    println!("Node ID: {}", core.node_id());
    println!("Sending: {}", path);
    println!("Generating ticket...\n");

    let progress = ConsoleProgress::new();
    match core.send(path, Some(Box::new(progress))) {
        Ok(ticket) => {
            println!("\n🎫 Ticket (share this with recipient):");
            println!("{}", ticket);

            if listen {
                println!("\n🔊 Listening for connections... Press Ctrl+C to stop.");
                let (tx, rx) = std::sync::mpsc::channel();
                ctrlc::set_handler(move || {
                    tx.send(()).ok();
                })
                .expect("Error setting Ctrl-C handler");
                rx.recv().expect("Error waiting for signal");
                println!("\n👋 Shutting down.");
            }
        }
        Err(e) => {
            eprintln!("Send failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_receive(ticket: String, dest: String, data_dir: String) {
    let core = Core::new(data_dir).expect("Failed to initialize Seyfr core");
    println!("Node ID: {}", core.node_id());
    println!("Receiving to: {}\n", dest);

    let progress = ConsoleProgress::new();
    match core.receive(ticket, dest, Some(Box::new(progress))) {
        Ok(_) => {
            println!("\nTransfer complete.");
        }
        Err(e) => {
            eprintln!("Receive failed: {}", e);
            std::process::exit(1);
        }
    }
}
