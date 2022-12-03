use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser)]
#[command(
    name = "RIP [ Rust Indexer for Plex ]",
    version = "0.0.1",
    author = "Hunt0k4r"
)]
struct Args {
    #[arg(long, short = 'd', required = false)]
    path_to_base_dir: String,

    #[arg(long, short = 't', required = false)]
    path_to_file_tree: String,

    #[arg(long, short = 'f', required = false)]
    path_to_destination: String,
}

fn main() {
    let args = Args::parse();
    let file_tree = FileTree::new(args.path_to_base_dir);

    // // save file tree to file
    // fs::write("file_tree.json", file_tree.get_json_string()).unwrap();

    let formated_file_tree = file_tree.get_formatted_file_tree();
    println!("{}", formated_file_tree);
}

#[derive(Serialize, Deserialize, Debug)]
struct FileTree {
    path: String,
    files: Vec<String>,
    directories: Vec<FileTree>,
}

impl FileTree {
    /*
    Returns the struct FileTree with all directories and files in the file tree
    Example:
    path = "C:/Users/username/Documents"
    FileTree = {
        path: "C:/Users/username/Documents",
        files: ["file1.txt", "file2.txt", "file3.txt"],
        directories: [
            {
                path: "C:/Users/username/Documents/Directory1",
                files: ["file1.txt", "file2.txt", "file3.txt"],
                directories: []
            },
            {
                path: "C:/Users/username/Documents/Directory2",
                files: ["file1.txt", "file2.txt", "file3.txt"],
                directories: []
            }
        ]
    }
     */
    fn new(path: String) -> FileTree {
        let mut directories = Vec::new();
        let mut files = Vec::new();

        for entry in fs::read_dir(path.clone()).unwrap() {
            let entry = entry.unwrap().path().display().to_string();
            if fs::metadata(entry.clone()).unwrap().is_dir() {
                directories.push(FileTree::new(entry));
            } else {
                files.push(entry);
            }
        }

        FileTree {
            path,
            files,
            directories,
        }
    }

    /*
    Returns a vector of all files in the file tree recursively
    Example:
    FileTree {
        path: "/home/user",
        files: ["file1", "file2"],
        directories: [
            FileTree {
                path: "/home/user/dir1",
                files: ["file3", "file4"],
                directories: []
            },
            FileTree {
                path: "/home/user/dir2",
                files: ["file5", "file6"],
                directories: []
            }
        ]
    }
    returns: ["/home/user/file1", "/home/user/file2", "/home/user/dir1/file3", "/home/user/dir1/file4", "/home/user/dir2/file5", "/home/user/dir2/file6"]
    */
    fn get_directories_list(self) -> Vec<String> {
        // create the vector
        let mut directories_list: Vec<String> = Vec::new();
        // concatenate the files in the current directory to the vector
        directories_list.extend(self.files);
        // concatenate the files in the subdirectories to the vector
        for directory in self.directories {
            directories_list.extend(directory.get_directories_list());
        }
        // return the vector
        return directories_list;
    }

    /*
    Returns a formated string of the file tree
    Example:
    FileTree {
        path: "/home/user",
        files: ["file1", "file2"],
        directories: [
            FileTree {
                path: "/home/user/dir1",
                files: ["file3", "file4"],
                directories: []
            },
            FileTree {
                path: "/home/user/dir2",
                files: ["file5", "file6"],
                directories: []
            }
        ]
    }
    returns: "
    home
    ├── user
    │   ├── file1
    │   ├── file2
    │   ├── dir1
    │   │   ├── file3
    │   │   └── file4
    │   └── dir2
    │       ├── file5
    │       └── file6
    "
    */
    // add the number of spaces to the string with default value 0
    fn get_formatted_file_tree(self) -> String {
        let mut formated_file_tree: String = String::new(); // create the string
        formated_file_tree.push_str(&self.path); // add the path to the string

        if self.files.iter().len() > 0 {
            for file in self.files.iter().take(self.files.len() - 1) {
                let num_of_spaces = self.path.split("/").count() - 1; // calculate the number of spaces
                let file = file.replace(&self.path, ""); // remove path from file
                formated_file_tree.push_str(&format!(
                    "\n{}├── {}",
                    "  ".repeat(num_of_spaces),
                    file
                ));
            }

            if self.files.iter().len() > 0 {
                let num_of_spaces = self.path.split("/").count() - 1; // calculate the number of spaces
                let file = self.files.last().unwrap().replace(&self.path, ""); // remove path from file
                formated_file_tree.push_str(&format!(
                    "\n{}└── {}",
                    "  ".repeat(num_of_spaces),
                    file
                ));
            }
        }

        for directory in self.directories {
            let num_of_spaces = self.path.split("/").count() - 1; // calculate the number of spaces
            formated_file_tree.push_str(&format!(
                "\n{}{}",
                "  ".repeat(num_of_spaces),
                directory.get_formatted_file_tree()
            ));
        }

        return formated_file_tree;
    }

    /*
    Returns the JSON representation of the file tree
    Example:
    FileTree {
        path: "/home/user",
        files: ["file1", "file2"],
        directories: [
            FileTree {
                path: "/home/user/dir1",
                files: ["file3", "file4"],
                directories: []
            },
            FileTree {
                path: "/home/user/dir2",
                files: ["file5", "file6"],
                directories: []
            }
        ]
    }
    returns: {
        "path": "/home/user",
        "files": ["file1", "file2"],
        "directories": [
            {
                "path": "/home/user/dir1",
                "files": ["file3", "file4"],
                "directories": []
            },
            {
                "path": "/home/user/dir2",
                "files": ["file5", "file6"],
                "directories": []
            }
        ]
    }
    */
    fn get_json_string(self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        return json;
    }
}
