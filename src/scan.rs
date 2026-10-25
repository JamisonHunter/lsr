use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct Document {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub incomplete: bool,
}

pub struct ScanIssue {
    pub path: PathBuf,
    pub error: io::Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub bytes: u64,
    pub incomplete: bool,
}

/// Collects the entries directly inside `dir` without following symlinks.
///
/// A failure to read `dir` itself is fatal and returned as an error. Problems
/// with individual entries are recorded in `issues` and the entry is omitted.
pub fn collect_documents(dir: &Path, issues: &mut Vec<ScanIssue>) -> io::Result<Vec<Document>> {
    let entries = fs::read_dir(dir)?;
    let mut documents = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                issues.push(ScanIssue {
                    path: dir.to_path_buf(),
                    error,
                });
                break;
            }
        };

        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(ScanIssue { path, error });
                continue;
            }
        };

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        documents.push(Document {
            name,
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            incomplete: false,
            path,
        });
    }

    Ok(documents)
}

/// Recursively totals the logical lengths of files under `path`.
///
/// Errors do not discard bytes that were counted successfully. Instead, each
/// problem is pushed to `issues`, and the returned result is marked incomplete.
/// Symlinks are never followed.
pub fn scan_dir(path: &Path, issues: &mut Vec<ScanIssue>) -> ScanResult {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            issues.push(ScanIssue {
                path: path.to_path_buf(),
                error,
            });
            return ScanResult {
                bytes: 0,
                incomplete: true,
            };
        }
    };

    let mut bytes = 0u64;
    let mut incomplete = false;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                issues.push(ScanIssue {
                    path: path.to_path_buf(),
                    error,
                });
                incomplete = true;
                break;
            }
        };

        let entry_path = entry.path();
        // DirEntry::metadata does not follow symlinks.
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(ScanIssue {
                    path: entry_path,
                    error,
                });
                incomplete = true;
                continue;
            }
        };

        if metadata.is_dir() {
            let result = scan_dir(&entry_path, issues);
            bytes = bytes.saturating_add(result.bytes);
            incomplete |= result.incomplete;
        } else {
            bytes = bytes.saturating_add(metadata.len());
        }
    }

    ScanResult { bytes, incomplete }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("lsr-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dirs: &[&Path]) {
        for dir in dirs {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn scan_dir_empty_directory_is_complete_and_zero() {
        let dir = temp_dir("empty");
        let mut issues = Vec::new();

        let result = scan_dir(&dir, &mut issues);

        assert_eq!(
            result,
            ScanResult {
                bytes: 0,
                incomplete: false
            }
        );
        assert!(issues.is_empty());
        cleanup(&[&dir]);
    }

    #[test]
    fn scan_dir_sums_nested_files() {
        let dir = temp_dir("nested");
        fs::write(dir.join("a.txt"), vec![0u8; 100]).unwrap();
        fs::create_dir_all(dir.join("sub/deeper")).unwrap();
        fs::write(dir.join("sub/b.txt"), vec![0u8; 250]).unwrap();
        fs::write(dir.join("sub/deeper/c.txt"), vec![0u8; 50]).unwrap();

        let mut issues = Vec::new();
        let result = scan_dir(&dir, &mut issues);

        assert_eq!(
            result,
            ScanResult {
                bytes: 400,
                incomplete: false
            }
        );
        assert!(issues.is_empty());
        cleanup(&[&dir]);
    }

    #[test]
    fn scan_dir_missing_directory_is_incomplete() {
        let dir = env::temp_dir().join(format!("lsr-test-does-not-exist-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let mut issues = Vec::new();
        let result = scan_dir(&dir, &mut issues);

        assert_eq!(
            result,
            ScanResult {
                bytes: 0,
                incomplete: true
            }
        );
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, dir);
    }

    #[test]
    fn scan_dir_on_a_regular_file_is_incomplete() {
        let dir = temp_dir("not-a-directory");
        let file = dir.join("file.txt");
        fs::write(&file, b"content").unwrap();

        let mut issues = Vec::new();
        let result = scan_dir(&file, &mut issues);

        assert_eq!(
            result,
            ScanResult {
                bytes: 0,
                incomplete: true
            }
        );
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, file);
        cleanup(&[&dir]);
    }

    #[cfg(unix)]
    #[test]
    fn scan_dir_retains_partial_totals_when_a_child_is_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_dir("partial");
        fs::write(dir.join("good.txt"), vec![0u8; 25]).unwrap();
        let locked = dir.join("locked");
        fs::create_dir(&locked).unwrap();
        fs::write(locked.join("secret.txt"), vec![0u8; 100]).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        // Root and some filesystems bypass these permissions; skip rather than
        // assert behavior the environment cannot exercise.
        if fs::read_dir(&locked).is_ok() {
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
            cleanup(&[&dir]);
            return;
        }

        let mut issues = Vec::new();
        let result = scan_dir(&dir, &mut issues);

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        cleanup(&[&dir]);

        assert_eq!(result.bytes, 25);
        assert!(result.incomplete);
        assert_eq!(issues.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn collect_documents_does_not_follow_top_level_symlinks() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("top-level-symlink");
        let target = temp_dir("top-level-symlink-target");
        fs::write(target.join("large.bin"), vec![0u8; 4_096]).unwrap();
        fs::write(dir.join("small.bin"), vec![0u8; 10]).unwrap();
        let link = dir.join("link-to-target");
        symlink(&target, &link).unwrap();

        let mut issues = Vec::new();
        let documents = collect_documents(&dir, &mut issues).unwrap();
        assert!(issues.is_empty());

        let link_document = documents
            .iter()
            .find(|document| document.name == "link-to-target")
            .unwrap();
        assert!(!link_document.is_dir);
        assert_eq!(
            link_document.size,
            fs::symlink_metadata(&link).unwrap().len()
        );

        let mut total = 0u64;
        for document in &documents {
            if document.is_dir {
                total += scan_dir(&document.path, &mut issues).bytes;
            } else {
                total += document.size;
            }
        }

        assert_eq!(total, 10 + fs::symlink_metadata(&link).unwrap().len());
        assert!(issues.is_empty());
        cleanup(&[&dir, &target]);
    }

    #[cfg(unix)]
    #[test]
    fn scan_dir_does_not_follow_nested_symlinks() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("nested-symlink");
        let target = temp_dir("nested-symlink-target");
        fs::write(target.join("large.bin"), vec![0u8; 4_096]).unwrap();
        fs::write(dir.join("small.bin"), vec![0u8; 10]).unwrap();
        let link = dir.join("link-to-target");
        symlink(&target, &link).unwrap();

        let mut issues = Vec::new();
        let result = scan_dir(&dir, &mut issues);

        assert!(!result.incomplete);
        assert!(issues.is_empty());
        assert_eq!(
            result.bytes,
            10 + fs::symlink_metadata(&link).unwrap().len()
        );
        cleanup(&[&dir, &target]);
    }
}
