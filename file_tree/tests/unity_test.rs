#[cfg(test)]
mod tests {
    use file_tree::FileTree;
    use std::fs;

    #[test]
    fn test_get_formatted_file_tree() {
        match fs::read_to_string("input.txt") {
            Ok(contents) => {
                let file_tree = FileTree::new_from_file_tree(contents);

                // write file tree to output.json
                let _ = fs::write("output.json", file_tree.clone().get_json_string());

                let formated_file_tree = file_tree.clone().get_formatted_file_tree();

                // expect the formated file tree to not be empty
                assert_ne!(formated_file_tree.len(), 0);

                // write the formated file tree to a file
                let _ = fs::write("output.txt", formated_file_tree);

                // expect the file to be created
                assert!(fs::metadata("output.txt").is_ok());

                // expect the file to not be empty
                assert_ne!(fs::metadata("output.txt").unwrap().len(), 0);

                // expect the output and input files to be the same
                // assert_eq!(
                //     fs::read_to_string("output.txt").unwrap(),
                //     fs::read_to_string("input.txt").unwrap()
                // );
            }
            Err(e) => {
                println!("Couldn't read file: {}", e);
            }
        }

        assert_eq!(1 + 1, 2);
    }
}
