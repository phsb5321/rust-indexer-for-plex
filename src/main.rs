use crate::entities::file_tree::FileTree;
use clap::Parser;

pub mod entities;

#[derive(Parser)]
#[command(
    name = "RIP [ Rust Indexer for Plex ]",
    version = "0.0.1",
    author = "Hunt0k4r"
)]
struct Args {
    #[arg(long, short = 'd', required = true)]
    path_to_base_dir: String,
    // #[arg(long, short = 't', required = false)]
    // path_to_file_tree: String,

    // #[arg(long, short = 'f', required = false)]
    // path_to_destination: String,
}

fn main() {
    let args = Args::parse();
    let file_tree = FileTree::new(args.path_to_base_dir);

    // // save file tree to file
    // fs::write("file_tree.json", file_tree.get_json_string()).unwrap();

    let formated_file_tree = file_tree.get_formatted_file_tree();
    println!("{}", formated_file_tree);
}
