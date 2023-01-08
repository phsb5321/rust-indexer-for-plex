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

        #[arg(long, short = 'p', required = false)]
        use_plex_folder_structure: bool,
    },
}

fn main() {
    let args = Args::parse();

    match args.action {
        Action::SymLink {
            path_to_base_dir,
            path_to_destination,
            use_plex_folder_structure,
        } => {
            println!("SymLinking {} to {}", path_to_base_dir, path_to_destination);
            let file_tree = FileTree::new_from_directory(path_to_base_dir);
            match use_plex_folder_structure {
                true => {
                    println!("Using PLEX Folder Structure");
                    file_tree.plex_course_sym_link(path_to_destination)
                }
                false => {
                    println!("Using Default Folder Structure");
                    file_tree.sym_link(path_to_destination)
                }
            }
        }
    }
}
