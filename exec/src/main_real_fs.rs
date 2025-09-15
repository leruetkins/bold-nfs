use bold::ServerBuilder;
use clap::Parser;
use vfs::PhysicalFS;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the directory to share
    #[arg(required = true)]
    path: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let root_path = &cli.path;
    println!("Sharing directory: {}", root_path);

    let fs = PhysicalFS::new(root_path);
    let root = fs.into();

    let server = ServerBuilder::new(root).bind("0.0.0.0:11112").build();
    server.start();
}