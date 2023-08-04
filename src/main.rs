use clap::{Parser, Subcommand};
use file_tree::FileTree;

#[derive(Parser)]
#[command(
    name = "RIP [ Rust Indexer for Plex ]",
    version = "0.0.1",
    author = "Hunt0k4r"
)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    #[command(name = "sym-link")]
    SymLink {
        #[arg(long, short = 'd', required = true)]
        path_to_base_dir: String,

        #[arg(long, short = 'f', required = false)]
        path_to_destination: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.action {
        Action::SymLink {
            path_to_base_dir,
            path_to_destination,
        } => {
            println!("SymLinking {} to {}", path_to_base_dir, path_to_destination);
            let file_tree = FileTree::from_directory(path_to_base_dir.to_string(), 1);
            file_tree.create_grouped_symlinks(path_to_destination.to_string());
        }
    }
}
