use super::file_tree::FileTree;
use std::fs;

pub struct PlexCourse {
    pub base_dir: FileTree,
    pub file_tree: FileTree,
    pub destination: String,
}

impl PlexCourse {
    /// Returns a new PlexCourse struct
    ///
    /// Example:
    /// ```rust
    /// let plex_course = PlexCourse::new(
    ///     FileTree::new("C:/Users/username/Documents".to_string()),
    ///     FileTree::new("C:/Users/username/Documents".to_string()),
    ///     "C:/Users/username/Documents".to_string()
    /// );
    /// ```
    ///
    /// # Arguments
    /// * `base_dir` - The base directory of the course
    /// * `file_tree` - The file tree of the course
    /// * `destination` - The destination of the course
    ///
    /// # Panics
    /// Panics if the base_dir, file_tree, or destination are not valid paths
    ///
    /// # Errors
    pub fn new(base_dir: FileTree, file_tree: FileTree, destination: String) -> PlexCourse {
        PlexCourse {
            base_dir,
            file_tree,
            destination,
        }
    }

    /// Generates the Symbolic Links for the base directory using the file tree and destination with the season and episode numbers
    ///
    /// Example:
    /// ```rust
    /// let plex_course = PlexCourse::new(
    ///     FileTree::new("C:/Users/username/Documents".to_string()),
    ///     FileTree::new("C:/Users/username/Documents".to_string()),
    ///     "C:/Users/username/Documents".to_string()
    /// );
    /// plex_course.generate_symbolic_links();
    /// ```
    pub fn generate_symbolic_links(self) {
        let mut season_number = 1;
        let mut episode_number = 1;

        let directory_file_list = self.base_dir.get_directories_list();
        let file_tree_file_list = self.file_tree.get_directories_list();

        for directory in directory_file_list {
            let mut season_path = self.destination.clone();
            season_path.push_str(&format!("\\Season {}", season_number));
            fs::create_dir(season_path).unwrap();

            for file in file_tree_file_list.clone() {
                if file.contains(&directory) {
                    let mut episode_path = self.destination.clone();
                    episode_path.push_str(&format!(
                        "\\Season {}\\Episode {}.mp4",
                        season_number, episode_number
                    ));
                    fs::create_dir(episode_path).unwrap();
                    episode_number += 1;
                }
            }
            season_number += 1;
        }
    }
}
