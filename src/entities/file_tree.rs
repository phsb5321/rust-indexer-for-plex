use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileTree {
    pub path: String,
    pub files: Vec<String>,
    pub directories: Vec<FileTree>,
}

impl FileTree {
    /// Returns the struct FileTree with all directories and files in the file tree
    ///
    /// Example:
    /// ```rust
    /// path = "C:/Users/username/Documents"
    /// FileTree = {
    ///     path: "C:/Users/username/Documents",
    ///     files: ["file1.txt", "file2.txt", "file3.txt"],
    ///     directories: [
    ///         {
    ///             path: "C:/Users/username/Documents/Directory1",
    ///             files: ["file1.txt", "file2.txt", "file3.txt"],
    ///             directories: []
    ///         },
    ///         {
    ///             path: "C:/Users/username/Documents/Directory2",
    ///             files: ["file1.txt", "file2.txt", "file3.txt"],
    ///             directories: []
    ///         }
    ///     ]
    /// }
    /// ```
    pub fn new(path: String) -> FileTree {
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

    /// Returns a struct FileTree with all directories and files in the file tree from a String vector
    ///
    /// Example:
    ///```
    /// values = [
    ///     "C:/Users/username/Documents",
    ///     "C:/Users/username/Documents/Directory1",
    ///     "C:/Users/username/Documents/Directory2",
    ///     "C:/Users/username/Documents/Directory1/file1.txt",
    ///     "C:/Users/username/Documents/Directory1/file2.txt",
    ///     "C:/Users/username/Documents/Directory1/file3.txt",
    /// ]
    ///
    /// FileTree = {
    ///     path: "C:/Users/username/Documents",
    ///     files: ["file1.txt", "file2.txt", "file3.txt"],
    ///     directories: [
    ///         {
    ///             path: "C:/Users/username/Documents/Directory1",
    ///             files: ["file1.txt", "file2.txt", "file3.txt"],
    ///             directories: []
    ///         },
    ///         {
    ///             path: "C:/Users/username/Documents/Directory2",
    ///             files: [],
    ///             directories: []
    ///         }
    ///     ]
    /// }
    ///```
    pub fn new_from_string_vector(values: Vec<String>) -> FileTree {
        assert!(values.len() > 0); // Expect at least one value

        let mut directories = Vec::new();
        let mut files = Vec::new();

        // Order the vector by length
        let mut values = values;
        values.sort_by(|a, b| a.len().cmp(&b.len()));

        // Use the first value as the root path ad remove it from the vector
        // if the root path ends with a slash, remove it
        let root_path = values.remove(0);
        let root_path = if root_path.ends_with("/") {
            root_path[..root_path.len() - 1].to_string()
        } else {
            root_path
        };

        // Iterate over the vector and add the values to the FileTree
        for entry in values.clone() {
            let mut clone_of_values = values.clone(); // Clone the vector to prevent borrowing issues

            let entry_clone = entry.replace(&root_path, ""); // Remove the root path from the entry
            let entry_split: Vec<&str> = entry_clone
                .split("/") // Split the entry by slashes
                .into_iter() // Convert the iterator to a vector
                .filter(|&x| x != "") // Remove empty strings
                .collect(); // Collect the iterator to a vector

            // If the entry is a file, add it to the files vector
            if entry_split.len() == 1 {
                files.push(entry);
            } else {
                let directory = entry_split[0].to_string(); // Get the directory name

                // Check if the directory is already in the directories vector
                if !directories
                    .iter()
                    .any(|x: &FileTree| x.path == format!("{}/{}", root_path, directory))
                {
                    let next_root_dix = format!("{}/{}", root_path, directory); // Get the next root path

                    // if the root dir does not start with a slash, add one
                    let next_root_dix = if !next_root_dix.starts_with("/") {
                        format!("/{}", next_root_dix)
                    } else {
                        next_root_dix
                    };

                    clone_of_values.push(next_root_dix); // Add the directory to the vector

                    // If the directory is not in the directories vector, add it
                    directories.push(FileTree::new_from_string_vector(
                        clone_of_values
                            .into_iter()
                            .filter(|x| x.contains(&directory))
                            .collect(),
                    ));
                }
            }
        }

        return FileTree {
            path: root_path,
            files,
            directories,
        };
    }

    ///Returns a vector of all files in the file tree recursively
    ///
    ///Example:
    ///```rust
    ///FileTree: {
    ///    path: "/home/user",
    ///    files: ["file1", "file2"],
    ///    directories: [
    ///        FileTree: {
    ///            path: "/home/user/dir1",
    ///            files: ["file3", "file4"],
    ///            directories: []
    ///        },
    ///        FileTree: {
    ///            path: "/home/user/dir2",
    ///            files: ["file5", "file6"],
    ///            directories: []
    ///        }
    ///    ]
    ///}
    ///returns: [
    ///     "/home/user/file1",
    ///     "/home/user/file2",
    ///     "/home/user/dir1/file3",
    ///     "/home/user/dir1/file4",
    ///     "/home/user/dir2/file5",
    ///     "/home/user/dir2/file6"
    /// ]
    /// ```
    pub fn get_directories_list(self) -> Vec<String> {
        let mut directories_list: Vec<String> = Vec::new(); // create the vector
        directories_list.extend(self.files); // concatenate the files in the current directory to the vector
        for directory in self.directories {
            directories_list.extend(directory.get_directories_list());
        } // concatenate the files in the subdirectories to the vector
        return directories_list;
    }

    /// Returns a formated string of the file tree
    ///
    /// Example:
    ///
    /// FileTree:
    /// ```
    /// {
    ///     path: "/home/user",
    ///     files: ["file1", "file2"],
    ///     directories: [
    ///         FileTree {
    ///             path: "/home/user/dir1",
    ///             files: ["file3", "file4"],
    ///             directories: []
    ///         },
    ///         FileTree {
    ///             path: "/home/user/dir2",
    ///             files: ["file5", "file6"],
    ///             directories: []
    ///         }
    ///     ]
    /// }
    /// ```
    /// returns:
    /// ```
    ///     home
    ///      ├── user
    ///      │   ├── file1
    ///      │   ├── file2
    ///      │   ├── dir1
    ///      │   │   ├── file3
    ///      │   │   └── file4
    ///      │   └── dir2
    ///      │       ├── file5
    ///      │       └── file6
    /// ```
    ///
    pub fn get_formatted_file_tree(self) -> String {
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

    ///Returns the JSON representation of the file tree
    ///Example:
    ///FileTree {
    ///    path: "/home/user",
    ///    files: ["file1", "file2"],
    ///    directories: [
    ///        FileTree {
    ///            path: "/home/user/dir1",
    ///            files: ["file3", "file4"],
    ///            directories: []
    ///        },
    ///        FileTree {
    ///            path: "/home/user/dir2",
    ///            files: ["file5", "file6"],
    ///            directories: []
    ///        }
    ///    ]
    ///}
    ///returns: {
    ///    "path": "/home/user",
    ///    "files": ["file1", "file2"],
    ///    "directories": [
    ///        {
    ///            "path": "/home/user/dir1",
    ///            "files": ["file3", "file4"],
    ///            "directories": []
    ///        },
    ///        {
    ///            "path": "/home/user/dir2",
    ///            "files": ["file5", "file6"],
    ///            "directories": []
    ///        }
    ///    ]
    ///}
    pub fn get_json_string(self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        return json;
    }

    /// Returns a 2D vector of all files in the file tree recursively
    /// Example:
    /// ```rust
    /// FileTree = {
    ///     path: "/home/user",
    ///     files: ["file1", "file2"],
    ///     directories: [
    ///         FileTree: {
    ///             path: "/home/user/dir1",
    ///             files: ["file3", "file4"],
    ///             directories: []
    ///         },
    ///         FileTree: {
    ///             path: "/home/user/dir2",
    ///             files: ["file5", "file6"],
    ///             directories: []
    ///         }
    ///     ]
    /// }
    /// ```
    /// returns:
    /// ```rust
    /// [
    ///     ["/home/user/file1", "/home/user/file2"],
    ///     ["/home/user/dir1/file3", "/home/user/dir1/file4"],
    ///     ["/home/user/dir2/file5", "/home/user/dir2/file6"]
    /// ]
    /// ```
    pub fn get_2d_vector(self) -> Vec<Vec<String>> {
        let mut directories_list: Vec<Vec<String>> = Vec::new(); // create the vector
        directories_list.push(self.files); // concatenate the files in the current directory to the vector
        for directory in self.directories {
            directories_list.push(directory.get_directories_list());
        } // concatenate the files in the subdirectories to the vector
        return directories_list;
    }
}
