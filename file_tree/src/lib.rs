use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::{collections::HashSet, os::unix::fs::symlink};

const POST_FIXES: [&str; 1] = [".mp4"];

/// Structure representing the hierarchy of files and directories within a root path.
#[derive(Serialize, Deserialize, Debug)]
pub struct FileTree {
    pub path: String,
    pub files: Vec<String>,
    pub directories: Vec<FileTree>,
}

impl FileTree {
    /// Returns a new instance of FileTree for a specified path.
    pub fn new(path: String) -> FileTree {
        FileTree {
            path,
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    /// Converts the FileTree instance into a JSON string representation.
    pub fn to_json(&self) -> String {
        json!(self).to_string()
    }

    /// Clones the current FileTree instance.
    pub fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            files: self.files.clone(),
            directories: self.directories.iter().map(|dir| dir.clone()).collect(),
        }
    }

    /// Extracts the name of the FileTree instance from its path.
    pub fn name(&self) -> String {
        self.path.split("/").last().unwrap().to_string()
    }

    /// Determines if a line represents a file by counting depth and checking POST_FIXES.
    fn is_file_line(line: &str) -> bool {
        let depth = line.matches("│").count() + line.matches("   ").count();
        POST_FIXES.iter().any(|&x| line.ends_with(x)) && depth == 0
    }

    /// Constructs a new FileTree instance from a directory path.
    pub fn from_directory(path: String) -> Self {
        let entries = fs::read_dir(&path).unwrap();
        let (files, dirs) = entries
            .map(|entry| entry.unwrap().path().display().to_string())
            .partition(|entry| fs::metadata(entry).unwrap().is_dir());

        Self {
            path,
            files,
            directories: dirs.into_iter().map(Self::from_directory).collect(),
        }
    }

    /// Constructs a new FileTree instance from a vector of string paths
    pub fn from_string_vector(mut values: Vec<String>) -> FileTree {
        assert!(!values.is_empty(), "Expect at least one value");

        let mut directories = Vec::new();
        let mut files = Vec::new();

        // Order the vector by path length
        values.sort_by_key(|a| a.len());

        // Use the first value as the root path and remove it from the vector
        // Remove trailing slash if present
        let root_path = values.remove(0).trim_end_matches('/').to_string();

        // Iterate over the vector and add the paths to the FileTree
        for entry in &values {
            let entry_clone = entry.replace(&root_path, ""); // Remove the root path from the entry
            let entry_split: Vec<&str> = entry_clone
                .split('/') // Split the entry by slashes
                .filter(|x| !x.is_empty()) // Remove empty strings
                .collect();

            // If the entry is a file, add it to the files vector
            if entry_split.len() == 1 {
                files.push(entry.clone());
            } else {
                let directory = entry_split[0]; // Get the directory name

                // Check if the directory is already in the directories vector
                if directories
                    .iter()
                    .all(|x: &FileTree| x.path != format!("{}/{}", root_path, directory))
                {
                    // Construct the next root path
                    let next_root_dix =
                        format!("{}/{}", root_path, directory.trim_start_matches('/'));

                    // Filter values containing the directory and pass them to the recursive function
                    directories.push(FileTree::from_string_vector(
                        values
                            .iter()
                            .filter(|x| x.contains(&next_root_dix))
                            .cloned()
                            .collect(),
                    ));
                }
            }
        }

        FileTree {
            path: root_path,
            files,
            directories,
        }
    }

    /// Constructs a new FileTree instance from a file tree string
    pub fn from_file_tree(file_tree: String) -> Self {
        let mut lines: Vec<String> = file_tree.lines().map(String::from).collect();

        let root_path = lines
            .remove(0)
            .replace("│   ", "")
            .replace("├── ", "")
            .replace("└── ", "");

        let mut new_tree = Self::new(root_path);

        let file_lines: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if Self::is_file_line(line) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        for &i in file_lines.iter().rev() {
            let line = lines.remove(i);
            new_tree.files.push(Self::extract_name(&line));
        }

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("├──") || line.starts_with("└──") {
                let dir = Self::from_file_tree(lines[i..].join("\n"));
                new_tree.directories.push(dir);
            }
        }

        new_tree
    }

    /// Helper function to extract name from a line
    fn extract_name(line: &str) -> String {
        line.replace("│   ", "")
            .replace("├── ", "")
            .replace("└── ", "")
    }

    /// Get a list of all files in the file tree by recursively traversing all directories.
    ///
    /// Each file's path is prefixed with the given prefix string.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current file tree.
    /// * `prefix` - The string to prefix all file paths with.
    pub fn to_file_list(&self, prefix: &str) -> Vec<String> {
        let mut files = vec![format!("{}{}", prefix, self.path.trim())];

        files.extend(
            self.files
                .iter()
                .map(|file| format!("{}{}", prefix, file.trim())),
        );

        files.extend(
            self.directories
                .iter()
                .flat_map(|directory| directory.to_file_list(&prefix)),
        );

        files
    }

    /// Convert the file tree to a string representation, with each file and directory on a new line.
    /// The root directory is only included if `root` is `true`.
    ///
    /// This function recursively processes all subdirectories.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current file tree.
    /// * `root` - Whether or not to include the root directory in the output.
    pub fn to_file_tree(&self, root: bool) -> String {
        let mut file_tree = if root {
            vec![self.path.clone()]
        } else {
            vec![]
        };

        file_tree.extend(self.files.iter().enumerate().map(|(i, file)| {
            let edge = if i == self.files.len() - 1 {
                "└── "
            } else {
                "├── "
            };
            format!("{}{}", edge, file)
        }));

        file_tree.extend(
            self.directories
                .iter()
                .enumerate()
                .flat_map(|(i, directory)| {
                    let (edge, prefix) = if i == self.directories.len() - 1 {
                        ("└── ", "    ")
                    } else {
                        ("├── ", "│   ")
                    };

                    let mut subtree = vec![format!("{}{}", edge, directory.path)];

                    subtree.extend(
                        directory
                            .to_file_tree(false)
                            .split("\n")
                            .map(|line| format!("{}{}", prefix, line)),
                    );

                    subtree
                }),
        );

        file_tree.join("\n")
    }

    /// Creates symbolic links for files by grouping them by seasons or chapters.
    ///
    /// # Arguments
    ///
    /// * `self` - An instance of the object where the function is called.
    /// * `destination` - A string representing the destination path for the symbolic links.
    /// * `grouping_type` - A string representing the type of grouping, either "Season" or "Chapter".
    pub fn create_grouped_symlinks(self, destination: String, grouping_type: &str) {
        // Retrieve and filter file list to include specific file types.
        let file_list = self
            .to_file_list("")
            .iter()
            .filter(|x| {
                !x.ends_with("/") && POST_FIXES.iter().any(|post_fix| x.ends_with(post_fix))
            })
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        // Create a sorted vector of unique group names extracted from file paths.
        let mut group_names: Vec<&str> = file_list
            .iter()
            .map(|file| file.rsplitn(2, "/").nth(1).unwrap())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        group_names.sort();

        // Iterate through each group, create a directory, and generate symbolic links.
        for (i, group) in group_names.iter().enumerate() {
            let group_dir = match grouping_type {
                "Season" => format!("Season {} - {}", i + 1, group),
                _ => group.to_string(),
            };

            if let Err(e) = fs::create_dir_all(format!("{}/{}", destination, group_dir)) {
                println!("Error creating directory: {} -> {}", group_dir, e);
                continue;
            }

            // Generate and sort group-specific file list.
            let mut group_files = file_list
                .iter()
                .filter(|file| file.contains(group))
                .map(|file| file.rsplit("/").collect::<Vec<&str>>())
                .collect::<Vec<_>>();
            group_files.sort_by_key(|file| file[0]);

            // Create symbolic links for each file in the group.
            for (j, file) in group_files.iter().enumerate() {
                let link_name = match grouping_type {
                    "Season" => {
                        format!("S{:02}E{:02} - {}", i + 1, j + 1, file[0].replace(" ", "."))
                    }
                    _ => file[0].to_string(),
                };

                if let Err(e) = symlink(
                    &file.join("/"),
                    format!("{}/{}/{}", destination, group_dir, link_name),
                ) {
                    println!("Error creating symbolic link: {} -> {}", link_name, e);
                }
            }
        }
    }
}
