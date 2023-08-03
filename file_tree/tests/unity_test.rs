#[cfg(test)]
mod tests {
    use file_tree::FileTree;
    use std::fs;

    #[test]
    fn test_get_formatted_file_tree() {
        match fs::read_to_string("tests/input.txt") {
            Ok(contents) => {
                let file_tree = FileTree::new_from_file_tree(contents);

                // Write the fille_tree to a file_tree.txt
                let _ = fs::write("tests/output.txt", file_tree.to_file_tree(true));

                // Write file list to a file_list.txt
                let _ = fs::write(
                    "tests/file_list.txt",
                    file_tree.to_file_list(&file_tree.path).join("\n"),
                );

                // expect the file to be created
                assert!(fs::metadata("tests/output.txt").is_ok());

                // expect the file to not be empty
                assert_ne!(fs::metadata("tests/output.txt").unwrap().len(), 0);

                // expect the output and input files to be the same
                assert_eq!(
                    fs::read_to_string("tests/output.txt").unwrap(),
                    fs::read_to_string("tests/input.txt").unwrap()
                );
            }
            Err(e) => {
                println!("Couldn't read file: {}", e);
            }
        }
    }
}
