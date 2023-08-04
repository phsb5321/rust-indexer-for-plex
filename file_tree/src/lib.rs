use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, read_dir, ReadDir};
use std::os::unix::fs::symlink;
use std::path::Path;

// Constant to store postfixes
const POST_FIXES: [&str; 1] = [".mp4"];

// Create an enum to store the grouping type
pub enum GroupingType {
    Plex,
    Original,
}

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

    /// Partition the entries of a directory into files and directories.
    ///
    /// This function takes a `ReadDir` iterator (which is a result of the `read_dir` function from `std::fs`)
    /// and returns a tuple of two `Vec<String>`. The first vector contains the paths to files and
    /// the second vector contains the paths to directories.
    pub fn partition_entries(entries: ReadDir) -> (Vec<String>, Vec<String>) {
        entries
            .filter_map(Result::ok) // Filter out errors
            .map(|entry| entry.path().display().to_string()) // Convert to string
            .partition(|entry| fs::metadata(entry).unwrap().is_dir()) // Partition into files and directories
    }

    /// Constructs a new instance of FileTree by reading and processing a directory path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to be processed.
    /// * `season` - The season number corresponding to the recursion depth.
    ///
    /// # Returns
    ///
    /// * A new instance of `Self` (FileTree) containing the file tree from the given directory.
    pub fn from_directory(path: String, season: usize) -> Self {
        let entries = fs::read_dir(&path).unwrap();
        let (files, dirs) = Self::partition_entries(entries);

        // Generate new file names with season and episode numbers
        let files = files
            .into_iter()
            .enumerate()
            .map(|(i, file)| {
                let file_stem = Path::new(&file)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                format!("S{:02}E{:02} - {}", season, i + 1, file_stem)
            })
            .collect();

        Self {
            path,
            files,
            directories: dirs
                .into_iter()
                .map(|dir| Self::from_directory(dir, season + 1))
                .collect(),
        }
    }

    pub fn create_grouped_symlinks(self, destination: String) {
        let new_path = Path::new(&destination);
        if !new_path.exists() {
            fs::create_dir_all(&new_path).unwrap();
        }

        for (i, file) in self.files.iter().enumerate() {
            let new_file_path = new_path.join(file);
            let old_file_path =
                Path::new(&self.path).join(format!("S{:02}E{:02} - {}", 1, i + 1, file));

            if let Err(error) = std::os::unix::fs::symlink(&old_file_path, &new_file_path) {
                println!(
                    "Error creating symbolic link: {} -> {}",
                    new_file_path.display(),
                    error
                );
            }
        }

        for directory in self.directories {
            directory.create_grouped_symlinks(destination.clone());
        }
    }
}
