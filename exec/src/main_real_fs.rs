use std::path::PathBuf;

use bold::ServerBuilder;
use bold::vfs::{VfsPath, PhysicalFS, AltrootFS};
use clap::Parser;
use tracing::Level;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a directory to serve
    directory: String,
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    }

    let directory = cli.directory;
    let path = PathBuf::from(directory);

    println!("Serving directory: {:?}", path);

    // Create a VfsPath from the physical filesystem path
    let physical_fs = PhysicalFS::new(path);
    let root: VfsPath = AltrootFS::new(physical_fs.into()).into();
    
    // Создаем NFS сервер с реальной файловой системой
    let server = ServerBuilder::new(root)
        .bind("0.0.0.0:2049")
        .build();
    server.start();
}