use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, os::unix::fs::symlink};
const POST_FIXES: [&str; 3] = [".mp4", ".zip", ".ts"];

#[derive(Serialize, Deserialize, Debug)]
pub struct FileTree {
    pub path: String,
    pub files: Vec<String>,
    pub directories: Vec<FileTree>,
}

impl FileTree {
    pub fn new(path: String) -> FileTree {
        FileTree {
            path,
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    pub fn new_from_real_directory(path: String) -> FileTree {
        let mut directories = Vec::new();
        let mut files = Vec::new();

        for entry in fs::read_dir(path.clone()).unwrap() {
            let entry = entry.unwrap().path().display().to_string();
            if fs::metadata(entry.clone()).unwrap().is_dir() {
                directories.push(FileTree::new_from_real_directory(entry));
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

    fn is_line_a_file(line: String) -> bool {
        // Use Regex to count the amount of "│"
        let depth_by_poll = Regex::new(r"│").unwrap().find_iter(line.as_str()).count();

        // Use Regex to count the amount of "    "
        let depth_by_tab = Regex::new(r"    ")
            .unwrap()
            .find_iter(line.as_str())
            .count();

        // Sum the depth
        let depth = depth_by_poll + depth_by_tab;

        // Test the line if it ends with one of POST_FIXES
        if POST_FIXES.iter().any(|&x| line.ends_with(x)) && depth == 0 {
            return true;
        }

        return false;
    }

    pub fn new_from_file_tree(file_tree: String) -> FileTree {
        // First create an array with the lines of the file tree
        let mut file_tree_lines: Vec<String> = file_tree.lines().map(|x| x.to_string()).collect();

        // Then, get the root path that is the first line by splicing the array
        let root_path = file_tree_lines[0]
            .to_string()
            .replace("│   ", "")
            .replace("├── ", "")
            .replace("└── ", "");
        file_tree_lines.remove(0); // Remove the root path from the array

        // Second create a new FileTree with the root path
        let mut file_tree = FileTree::new(root_path.to_string());

        let mut index_list_to_trash: Vec<usize> = vec![];

        // Removes the file of a given file tree
        for (i, line) in file_tree_lines.iter_mut().enumerate() {
            if FileTree::is_line_a_file(line.to_string()) {
                // Get the file name
                let file_name = line
                    .to_string()
                    .replace("│   ", "")
                    .replace("├── ", "")
                    .replace("└── ", "");

                // Add the file to the file tree
                file_tree.files.push(file_name);

                // Add the index to the list of indexes to trash
                index_list_to_trash.push(i);
            }
        }

        // Remove the files from the file_tree_lines vector
        for index in index_list_to_trash.iter().rev() {
            file_tree_lines.remove(*index);
        }

        let mut avoid_lines_index: Vec<usize> = vec![];

        // Third, iterate over all the other lines removing the first 4 characters
        for (i, line) in file_tree_lines.clone().iter_mut().enumerate() {
            if avoid_lines_index.contains(&i) {
                continue;
            }

            if line.starts_with("├──") || line.starts_with("└──") {
                // Create a new vector to hold the lines of the directory
                let mut directory_lines: Vec<String> = Vec::new();

                // Iterate over the rest of the lines and make them Strings
                for (_, forward_line) in file_tree_lines.clone()[i + 1..].iter().enumerate() {
                    let depth = Regex::new(r"│")
                        .unwrap()
                        .find_iter(forward_line.as_str())
                        .count();

                    if (forward_line.to_string().starts_with("├──")
                        || forward_line.to_string().starts_with("└──"))
                        && depth == 0
                    {
                        break;
                    }

                    // Remove the first occurrence of "│  " from the line
                    let line_to_push = forward_line.to_string().replacen("│   ", "", 1);

                    directory_lines.push(line_to_push);
                }

                // Push root path to the directory lines
                directory_lines.insert(
                    0,
                    file_tree_lines[i]
                        .to_string()
                        .replace("├── ", "")
                        .replace("└── ", ""),
                );

                // Instead of removing the lines, add the indexes to the avoid_lines_index vector
                for j in i..i + directory_lines.len() {
                    avoid_lines_index.push(j);
                }

                // Create a new FileTree from the directory lines
                let directory = FileTree::new_from_file_tree(directory_lines.join("\n"));

                // Add the directory to the directories vector
                file_tree.directories.push(directory);
            }
        }

        return file_tree;
    }

    pub fn get_directories_list(self) -> Vec<String> {
        // Create a new empty vector to hold the list of directories
        let mut directories_list: Vec<String> = Vec::new();

        // Concatenate the list of files in the current directory to the vector
        directories_list.extend(self.files);

        // Iterate through the list of subdirectories
        for directory in self.directories {
            // For each subdirectory, concatenate its list of files to the vector
            directories_list.extend(directory.get_directories_list());
        }

        // Return the final list of directories
        return directories_list;
    }

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

    pub fn get_json_string(self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        return json;
    }

    pub fn get_2d_vector(self) -> Vec<Vec<String>> {
        let mut directories_list: Vec<Vec<String>> = Vec::new(); // create the vector
        directories_list.push(self.files); // concatenate the files in the current directory to the vector
        for directory in self.directories {
            directories_list.push(directory.get_directories_list());
        } // concatenate the files in the subdirectories to the vector
        return directories_list;
    }

    pub fn clone(&self) -> FileTree {
        let mut new_file_tree = FileTree {
            path: self.path.clone(),
            files: self.files.clone(),
            directories: Vec::new(),
        };
        for directory in self.directories.iter() {
            new_file_tree.directories.push(directory.clone());
        }
        return new_file_tree;
    }

    pub fn get_name(&self) -> String {
        let path = self.path.clone();
        let path = path.split("/").collect::<Vec<&str>>();
        return path.last().unwrap().to_string();
    }

    pub fn generate_symbolic_links(self, destination: String, season_number: i32) {
        let name = self.get_name();

        let season = format!("Season {:02}", season_number);
        let path = format!("{}/{}", destination, season.as_str());

        match fs::create_dir(&path) {
            Ok(()) => println!("Directory created: {}", path),
            Err(err) => {
                println!("Error creating directory: {} -> {}", name, err);
                return;
            }
        }

        // create a file_list variable to fold the files filtered to mp4 and sorted
        let mut file_list = self
            .files
            .iter()
            .filter(|file| file.ends_with(".mp4"))
            .collect::<Vec<&String>>();

        // sort the files
        file_list.sort();

        // create a symbolic link for each file in the directory
        for (index, file) in file_list.iter().enumerate() {
            let file_name_array = file.split("/").collect::<Vec<&str>>();
            let file_name = file_name_array.last().unwrap();

            let prefix = format!("S01E{:02} - ", index + 1);
            match symlink(&file, format!("{}/{}{}", path, prefix, file_name)) {
                Ok(()) => println!("Symbolic link created: {}", file_name),
                Err(e) => println!("Error creating symbolic link: {} -> {}", file_name, e),
            }
        }

        // create a symbolic link for each subdirectory
        for directory in self.directories {
            directory.generate_symbolic_links(path.clone(), season_number + 1);
        }
    }
}
