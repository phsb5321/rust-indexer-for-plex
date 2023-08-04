use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, read_dir, ReadDir};
use std::os::unix::fs::symlink;
use std::path::Path;

// Constant to store postfixes
const POST_FIXES: [&str; 1] = [".mp4"];

/// Represents a tree structure for files
#[derive(Serialize, Deserialize, Debug)]
pub struct FileTree {
    pub path: String,
    pub files: Vec<String>,
    pub directories: Vec<FileTree>,
}

/// Struct FileTree Implementation
impl FileTree {
    /// Constructor for the FileTree struct. Initializes a new FileTree with
    /// the specified path. Note that the `files` and `directories` fields
    /// are initialized as empty vectors.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path to the file tree.
    ///
    /// # Returns
    ///
    /// * A new instance of `Self` (FileTree).
    pub fn new(path: String) -> Self {
        Self {
            path,
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    /// Converts the current FileTree instance to a JSON string.
    ///
    /// # Returns
    ///
    /// * A JSON-formatted string representation of the FileTree instance.
    pub fn to_json(&self) -> String {
        json!(self).to_string()
    }

    /// Returns the last component of the FileTree's path, effectively giving the name of the FileTree.
    ///
    /// # Returns
    ///
    /// * A string representing the name of the FileTree.
    pub fn name(&self) -> String {
        self.path.split("/").last().unwrap().to_string()
    }

    /// Constructs a new instance of FileTree by reading and processing a directory path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to be processed.
    ///
    /// # Returns
    ///
    /// * A new instance of `Self` (FileTree) containing the file tree from the given directory.
    pub fn from_directory(path: String) -> Self {
        let entries = fs::read_dir(&path).unwrap();
        let (files, dirs) = partition_entries(entries);
        Self {
            path,
            files,
            directories: dirs.into_iter().map(Self::from_directory).collect(),
        }
    }

    /// Constructs a new instance of FileTree by processing a vector of paths represented as strings.
    ///
    /// # Arguments
    ///
    /// * `values` - Vector of strings, each representing a path in the file tree.
    ///
    /// # Returns
    ///
    /// * A new instance of `Self` (FileTree) constructed from the provided paths.
    ///
    /// # Panics
    ///
    /// * If the `values` vector is empty.
    pub fn from_string_vector(values: Vec<String>) -> Self {
        assert!(!values.is_empty(), "Expect at least one value");
        let (root_path, values) = prepare_paths(values);
        let (files, directories) = process_paths(root_path.clone(), values);
        Self {
            path: root_path,
            files,
            directories,
        }
    }

    /// Constructs a new instance of FileTree by processing a string representation of a FileTree.
    ///
    /// # Arguments
    ///
    /// * `file_tree` - A string representing a file tree.
    ///
    /// # Returns
    ///
    /// * A new instance of `Self` (FileTree) constructed from the provided string.
    pub fn from_file_tree(file_tree: String) -> Self {
        let (root_path, lines) = prepare_file_tree_lines(file_tree);
        let (files, directories) = process_file_and_dir_lines(lines);
        Self {
            path: root_path,
            files,
            directories,
        }
    }

    /// Converts the FileTree to a vector of strings representing all files in the file tree.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A string to be prepended to each file in the list.
    ///
    /// # Returns
    ///
    /// * A vector of strings, each representing a file in the file tree, prefixed with `prefix`.
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

    /// Converts the FileTree to a string representation of the tree of files.
    ///
    /// # Arguments
    ///
    /// * `root` - A boolean indicating whether this FileTree instance is the root of the file tree.
    ///
    /// # Returns
    ///
    /// * A string representing the tree of files.
    pub fn to_file_tree(&self, root: bool) -> String {
        let mut file_tree = if root {
            vec![self.path.clone()]
        } else {
            vec![]
        };
        file_tree.extend(
            self.files
                .iter()
                .enumerate()
                .map(|(i, file)| format_file(i, file)),
        );
        file_tree.extend(
            self.directories
                .iter()
                .enumerate()
                .flat_map(|(i, directory)| format_directory(i, directory)),
        );
        file_tree.join("\n")
    }

    /// Creates symlinks for files grouped by the provided criteria, inside the provided destination directory.
    ///
    /// # Arguments
    ///
    /// * `self` - Consumes the instance, as we're done with it after this operation.
    /// * `destination` - The directory where the symlinks will be created.
    /// * `grouping_type` - The criteria to use for grouping the files.
    ///
    /// # Note
    ///
    /// This function only creates symlinks for files that have a postfix contained in the POST_FIXES array.
    pub fn create_grouped_symlinks(self, destination: String, grouping_type: &str) {
        // We generate a list of files which meet our criteria using depth-first search.
        let file_list: Vec<String> = generate_file_list(&Path::new(&self.path), &POST_FIXES);

        // We then generate the unique group names based on our file list.
        let group_names: Vec<String> = get_sorted_group_names(file_list.clone());

        // For each group, we create a directory in the destination, and create
        // symlinks for all the files in that group.
        for (group_index, group) in group_names.iter().enumerate() {
            // Formatting the name of the group directory with its index.
            let group_dir = format_group_dir(group_index, &group, grouping_type);

            // Attempt to create a new directory with the group directory name.
            // If this fails, log an error and skip to the next iteration of the loop.
            if let Err(error) = fs::create_dir_all(format!("{}/{}", destination, group_dir)) {
                println!("Error creating directory: {} -> {}", group_dir, error);
                continue;
            }

            // Retrieve the files that belong to the current group.
            let group_files = get_sorted_group_files(file_list.clone(), &group);

            // Create a symbolic link for each file in the group.
            for (file_index, file) in group_files.iter().enumerate() {
                // Format the name of the link with its index and the grouping type.
                let link_name = format_link_name(group_index, file_index, file, grouping_type);

                // Attempt to create the symlink. If this fails, log an error.
                if let Err(error) = symlink(
                    &file,
                    format!("{}/{}/{}", destination, group_dir, link_name),
                ) {
                    println!("Error creating symbolic link: {} -> {}", link_name, error);
                }
            }
        }
    }
}

// Helper functions

/// Partition the entries of a directory into files and directories.
///
/// This function takes a `ReadDir` iterator (which is a result of the `read_dir` function from `std::fs`)
/// and returns a tuple of two `Vec<String>`. The first vector contains the paths to files and
/// the second vector contains the paths to directories.
fn partition_entries(entries: ReadDir) -> (Vec<String>, Vec<String>) {
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path().display().to_string())
        .partition(|entry| fs::metadata(entry).unwrap().is_dir())
}

/// Prepares and sorts paths.
///
/// This function sorts a vector of paths by their length and removes the root path from the vector,
/// returning it alongside the modified vector.
fn prepare_paths(mut values: Vec<String>) -> (String, Vec<String>) {
    values.sort_by_key(|a| a.len());
    let root_path = values.remove(0).trim_end_matches('/').to_string();
    (root_path, values)
}

/// Processes paths into files and directories.
///
/// This function takes the root path and a vector of paths, and separates them into files and directories,
/// returning them as a tuple. The root path is used to differentiate between files and directories in a hierarchical manner.
fn process_paths(root_path: String, values: Vec<String>) -> (Vec<String>, Vec<FileTree>) {
    let mut directories = Vec::new();
    let mut files = Vec::new();

    for entry in &values {
        let entry_clone = entry.replace(&root_path, "");
        let entry_split: Vec<&str> = entry_clone.split('/').filter(|x| !x.is_empty()).collect();

        if entry_split.len() == 1 {
            files.push(entry.clone());
        } else {
            let directory = entry_split[0];
            if directories
                .iter()
                .all(|x: &FileTree| x.path != format!("{}/{}", root_path, directory))
            {
                let next_root_dix = format!("{}/{}", root_path, directory.trim_start_matches('/'));
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

    (files, directories)
}

/// Prepares lines from a FileTree represented as string.
///
/// This function takes a string representation of a FileTree and splits it into separate lines,
/// returning the root path and a vector of the other lines.
fn prepare_file_tree_lines(file_tree: String) -> (String, Vec<String>) {
    let mut lines: Vec<String> = file_tree.lines().map(String::from).collect();
    let root_path = lines.remove(0);
    (root_path, lines)
}

/// Processes lines into files and directories.
///
/// This function takes a vector of lines representing a file tree and separates them into files and directories,
/// returning them as a tuple.
fn process_file_and_dir_lines(lines: Vec<String>) -> (Vec<String>, Vec<FileTree>) {
    let (files, dirs) = filter_files(lines);
    let directories = process_directory_lines(dirs);
    (files, directories)
}

/// Filters lines into files and directories.
///
/// This function takes a vector of lines and separates them into files and directories, returning them as a tuple.
fn filter_files(lines: Vec<String>) -> (Vec<String>, Vec<String>) {
    lines
        .into_iter()
        .partition(|line| fs::metadata(line).unwrap().is_file())
}

/// Processes lines into directory FileTrees.
///
/// This function takes a vector of lines representing directories and transforms each one into a FileTree,
/// returning a vector of the results.
fn process_directory_lines(lines: Vec<String>) -> Vec<FileTree> {
    lines
        .into_iter()
        .filter(|line| fs::metadata(line).unwrap().is_dir())
        .map(|line| FileTree::from_directory(line))
        .collect()
}

/// Formats a file line with its index.
fn format_file(i: usize, file: &String) -> String {
    format!("File {}: {}", i + 1, file)
}

/// Formats a directory line with its index.
fn format_directory(i: usize, directory: &FileTree) -> Vec<String> {
    vec![format!(
        "Directory {}: {}",
        i + 1,
        directory.to_file_tree(false)
    )]
}

/// Gets sorted group names from a file list.
fn get_sorted_group_names(file_list: Vec<String>) -> Vec<String> {
    let mut group_names: Vec<String> = file_list
        .iter()
        .map(|file| file.split('/').last().unwrap().to_string())
        .collect();
    group_names.sort();
    group_names
}

/// Formats a group directory name with its index.
fn format_group_dir(i: usize, group: &str, grouping_type: &str) -> String {
    format!("{} {} - {}", grouping_type, i + 1, group)
}

/// Gets sorted group files from a file list.
fn get_sorted_group_files(file_list: Vec<String>, group: &str) -> Vec<String> {
    let mut group_files: Vec<String> = file_list
        .into_iter()
        .filter(|file| file.contains(group))
        .collect();
    group_files.sort();
    group_files
}

/// Formats a link name with its index.
fn format_link_name(i: usize, j: usize, file: &String, grouping_type: &str) -> String {
    let file_name = file.split('/').last().unwrap();
    format!(
        "{} {} - File {} - {}",
        grouping_type,
        i + 1,
        j + 1,
        file_name
    )
}

// The function generate_file_list is designed to generate a list of all files in a given directory structure.
// It operates recursively, so it's able to traverse subdirectories as well as the top level directory.
// Files are added to the list in a depth-first order, preserving the original order of files in each directory.
// The function only includes files whose names end with one of the specified postfixes.
fn generate_file_list(path: &Path, postfixes: &[&str]) -> Vec<String> {
    // Initialize an empty vector to store the file list.
    let mut file_list = vec![];

    // Check if the given path is a directory.
    if path.is_dir() {
        // If it is, attempt to read the directory. This returns a Result type which is an Ok variant if the operation succeeded,
        // and contains an iterator over the entries within the directory.
        if let Ok(entries) = read_dir(path) {
            // For each entry in the directory...
            for entry in entries {
                // Attempt to unwrap the entry. If it's valid (i.e., if this is an Ok variant of the Result type)...
                if let Ok(entry) = entry {
                    // Get the path to the entry.
                    let file_path = entry.path();

                    // Check if the file path ends with one of the specified postfixes and not ends with "/"
                    // This will be true for all valid files and false for directories and invalid files.
                    let is_valid_file = !file_path.ends_with("/")
                        && postfixes
                        .iter()
                        .any(|post_fix| file_path.ends_with(post_fix));

                    // If the entry is a valid file...
                    if is_valid_file {
                        // Convert the file path to a string and add it to the file list.
                        file_list.push(file_path.to_string_lossy().into_owned());
                    }
                    // If the entry is a directory...
                    else if file_path.is_dir() {
                        // Call generate_file_list recursively to get a list of files from the subdirectory,
                        // and add those files to the file list.
                        file_list.extend(generate_file_list(&file_path, postfixes));
                    }
                }
            }
        }
    }
    // If the given path is a file (not a directory)...
    else if path.is_file() {
        // Check if the file ends with one of the specified postfixes and not ends with "/"
        let is_valid_file =
            !path.ends_with("/") && postfixes.iter().any(|post_fix| path.ends_with(post_fix));

        // If the file is valid...
        if is_valid_file {
            // Convert the file path to a string and add it to the file list.
            file_list.push(path.to_string_lossy().into_owned());
        }
    }

    // Return the file list.
    file_list
}
