use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Recursively walks a directory and returns all file paths found.
pub fn walk_files(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut paths = Vec::new();
    _walk_files(&mut paths, dir)?;
    Ok(paths)
}

fn _walk_files(paths: &mut Vec<PathBuf>, path: &Path) -> Result<(), io::Error> {
    if path.is_dir() {
        // Iterate over directory entries
        for child in fs::read_dir(path)? {
            let child = child?;
            _walk_files(paths, &child.path())?;
        }
    } else {
        // Add the entry itself
        paths.push(path.to_path_buf());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_walk_dir() -> Result<(), io::Error> {
        // Create a temporary directory structure
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        // Create some nested directories and files
        let subdir1 = temp_path.join("subdir1");
        let subdir2 = temp_path.join("subdir1/subdir2");
        fs::create_dir(&subdir1)?;
        fs::create_dir(&subdir2)?;

        // Create some files
        fs::write(temp_path.join("file1.txt"), b"content1")?;
        fs::write(subdir1.join("file2.txt"), b"content2")?;
        fs::write(subdir2.join("file3.txt"), b"content3")?;

        // Test walk_dir
        let paths = walk_files(temp_path)?;

        // Check that we have the expected number of paths
        assert_eq!(paths.len(), 3); // root dir + 2 subdirs + 3 files

        // Check that specific paths exist in the result
        assert!(paths.contains(&temp_path.join("file1.txt")));
        assert!(paths.contains(&subdir1.join("file2.txt")));
        assert!(paths.contains(&subdir2.join("file3.txt")));

        Ok(())
    }
}
