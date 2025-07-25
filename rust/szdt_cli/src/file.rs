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

/// List all files in a directory (non-recursive).
pub fn list_files(path: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut paths = Vec::new();
    for child in fs::read_dir(path)? {
        let child = child?;
        if child.file_type()?.is_file() {
            paths.push(child.path());
        }
    }
    Ok(paths)
}

/// Write file to a path, creating parent directories if necessary.
pub fn write_file_deep<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    content: C,
) -> Result<(), io::Error> {
    let path = path.as_ref();
    let parent = path.parent().ok_or_else(|| io::Error::other("no parent"))?;
    fs::create_dir_all(parent)?;
    fs::write(path, content)
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

    #[test]
    fn test_write_file_deep() -> Result<(), io::Error> {
        // Create a temporary directory structure
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        // Write content to a new file in a new directory
        let new_file_path = temp_path.join("new_directory/new_file.txt");
        let new_file_content = b"new file content";

        write_file_deep(&new_file_path, new_file_content)?;

        // Check that file exists and content is as expected
        assert!(new_file_path.exists());
        assert_eq!(fs::read_to_string(new_file_path)?, "new file content");

        Ok(())
    }
}
