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
    // #[arg(long, short = 'd', required = true)]
    // path_to_base_dir: String,

    // #[arg(long, short = 't', required = false)]
    // path_to_file_tree: String,

    // #[arg(long, short = 'f', required = false)]
    // path_to_destination: String,
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
            let file_tree = FileTree::new(path_to_base_dir);
            file_tree.generate_symbolic_links(path_to_destination, 1);
        }
    }
}
