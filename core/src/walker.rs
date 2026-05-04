use std::path::{Path, PathBuf};

use crate::errors::SeyfrError;

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name() else {
        return false;
    };
    let bytes = name.as_encoded_bytes();
    bytes.first() == Some(&b'.') || name == std::ffi::OsStr::new("node_modules")
}

pub async fn collect_files(dir: &Path) -> Result<Vec<PathBuf>, SeyfrError> {
    let dir = dir.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let metadata = std::fs::metadata(&dir).map_err(SeyfrError::from)?;
        if !metadata.is_dir() {
            return Err(SeyfrError::NotADirectory {
                path: dir.to_string_lossy().to_string(),
            });
        }

        let mut files = Vec::new();
        let mut stack = vec![dir];

        while let Some(current) = stack.pop() {
            let entries = match std::fs::read_dir(&current) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries {
                let Ok(entry) = entry else { continue };
                let path = entry.path();

                let Ok(file_type) = entry.file_type() else {
                    continue;
                };

                if file_type.is_dir() {
                    if !should_skip_dir(&path) {
                        stack.push(path);
                    }
                } else if file_type.is_file() {
                    files.push(path);
                }
            }
        }

        files.sort();
        Ok(files)
    })
    .await
    .map_err(|e| SeyfrError::Internal {
        message: e.to_string(),
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_flat_sorted() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::write(dir.join("b.txt"), "").unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();
        fs::write(dir.join("c.txt"), "").unwrap();
        let files = collect_files(dir).await.unwrap();
        let names: Vec<_> = files.iter().map(|p| p.file_name().unwrap().to_str().unwrap().to_string()).collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[tokio::test]
    async fn test_nested() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::create_dir(dir.join("sub")).unwrap();
        fs::write(dir.join("root.txt"), "").unwrap();
        fs::write(dir.join("sub").join("nested.txt"), "").unwrap();
        let files = collect_files(dir).await.unwrap();
        let relative: Vec<_> = files.iter().map(|p| p.strip_prefix(dir).unwrap().to_str().unwrap().to_string()).collect();
        assert_eq!(relative, vec!["root.txt", Path::new("sub").join("nested.txt").to_str().unwrap()]);
    }

    #[tokio::test]
    async fn test_skips_hidden_dirs() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::create_dir(dir.join(".hidden")).unwrap();
        fs::write(dir.join("visible.txt"), "").unwrap();
        fs::write(dir.join(".hidden").join("secret.txt"), "").unwrap();
        let files = collect_files(dir).await.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|p| p.ends_with("visible.txt")));
        assert!(!files.iter().any(|p| p.ends_with("secret.txt")));
    }

    #[tokio::test]
    async fn test_skips_node_modules() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::create_dir(dir.join("node_modules")).unwrap();
        fs::write(dir.join("src.js"), "").unwrap();
        fs::write(dir.join("node_modules").join("pkg.js"), "").unwrap();
        let files = collect_files(dir).await.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|p| p.ends_with("src.js")));
        assert!(!files.iter().any(|p| p.ends_with("pkg.js")));
    }

    #[tokio::test]
    async fn test_empty_dir() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        let files = collect_files(dir).await.unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_nonexistent_dir() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_path_buf();
        drop(temp);
        let result = collect_files(&path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rejects_file() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("not_a_dir.txt");
        fs::write(&file_path, "").unwrap();
        let result = collect_files(&file_path).await;
        assert!(matches!(result, Err(SeyfrError::NotADirectory { .. })));
    }

    #[tokio::test]
    async fn test_root_hidden_dir_not_skipped() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join(".secrets");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("file.txt"), "").unwrap();
        let files = collect_files(&root).await.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|p| p.ends_with("file.txt")));
    }

    #[tokio::test]
    async fn test_skips_nested_hidden_and_node_modules() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::create_dir_all(dir.join("src").join(".git")).unwrap();
        fs::create_dir_all(dir.join("src").join("node_modules")).unwrap();
        fs::write(dir.join("src").join("main.rs"), "").unwrap();
        fs::write(dir.join("src").join(".git").join("HEAD"), "").unwrap();
        fs::write(dir.join("src").join("node_modules").join("lib.js"), "").unwrap();
        let files = collect_files(dir).await.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|p| p.ends_with("main.rs")));
        assert!(!files.iter().any(|p| p.ends_with("HEAD")));
        assert!(!files.iter().any(|p| p.ends_with("lib.js")));
    }

    #[tokio::test]
    async fn test_includes_hidden_files_at_root() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::write(dir.join(".gitignore"), "").unwrap();
        fs::write(dir.join("readme.txt"), "").unwrap();
        let files = collect_files(dir).await.unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with(".gitignore")));
        assert!(files.iter().any(|p| p.ends_with("readme.txt")));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_skips_symlinks() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        fs::write(dir.join("real.txt"), "").unwrap();
        symlink(dir.join("real.txt"), dir.join("link.txt")).unwrap();
        let files = collect_files(dir).await.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|p| p.ends_with("real.txt")));
        assert!(!files.iter().any(|p| p.ends_with("link.txt")));
    }

}
