use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait WallpaperOrdering: Send + Sync {
    fn next(&self, files: &[std::path::PathBuf]) -> Option<std::path::PathBuf>;
}

pub fn is_video_file(path: &Path, allow_animated: bool) -> bool {
    if !allow_animated {
        return false;
    }
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "mp4" | "webm" | "mov" | "avi" | "mkv")
    } else {
        false
    }
}

pub struct RandomOrdering;

impl WallpaperOrdering for RandomOrdering {
    fn next(&self, files: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
        if files.is_empty() {
            return None;
        }

        let index = rand::random_range(0..files.len());
        Some(files[index].clone())
    }
}

pub struct SequentialOrdering {
    index: AtomicUsize,
}

impl SequentialOrdering {
    pub fn new() -> Self {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}

impl Default for SequentialOrdering {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperOrdering for SequentialOrdering {
    fn next(&self, files: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
        if files.is_empty() {
            return None;
        }

        let index = self.index.fetch_add(1, Ordering::Relaxed) % files.len();
        Some(files[index].clone())
    }
}

pub fn get_image_files(dir: &Path, allow_animated: bool) -> Option<Vec<std::path::PathBuf>> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                let is_image = matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
                );
                let is_video = allow_animated
                    && matches!(ext.as_str(), "mp4" | "webm" | "mov" | "avi" | "mkv");
                is_image || is_video
            } else {
                false
            }
        })
        .map(|e| e.path())
        .collect();

    if files.is_empty() {
        return None;
    }

    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Some(files)
}

pub fn get_next_image(
    dir: &Path,
    strategy: &dyn WallpaperOrdering,
    allow_animated: bool,
) -> Option<String> {
    let files = get_image_files(dir, allow_animated)?;
    strategy
        .next(&files)
        .map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir(files: &[&str]) -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        for file in files {
            let path = dir.path().join(file);
            fs::write(&path, "test").unwrap();
        }
        dir
    }

    #[test]
    fn test_is_video_file_with_allow_animated_true() {
        let path = PathBuf::from("/test/video.mp4");
        assert!(is_video_file(&path, true));
    }

    #[test]
    fn test_is_video_file_with_allow_animated_false() {
        let path = PathBuf::from("/test/video.mp4");
        assert!(!is_video_file(&path, false));
    }

    #[test]
    fn test_is_video_file_image_extension() {
        let path = PathBuf::from("/test/image.jpg");
        assert!(!is_video_file(&path, true));
    }

    #[test]
    fn test_is_video_file_unsupported_extension() {
        let path = PathBuf::from("/test/file.txt");
        assert!(!is_video_file(&path, true));
    }

    #[test]
    fn test_get_image_files_with_animated_disabled() {
        let dir = create_test_dir(&["test.jpg", "video.mp4", "image.png"]);
        let files = get_image_files(dir.path(), false).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| {
            let ext = f.extension().unwrap().to_string_lossy();
            ext != "mp4"
        }));
    }

    #[test]
    fn test_get_image_files_with_animated_enabled() {
        let dir = create_test_dir(&["test.jpg", "video.mp4", "image.png"]);
        let files = get_image_files(dir.path(), true).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_get_image_files_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(get_image_files(dir.path(), true).is_none());
    }

    #[test]
    fn test_random_ordering() {
        let files = vec![PathBuf::from("1.jpg"), PathBuf::from("2.jpg")];
        let strategy = RandomOrdering;
        let result = strategy.next(&files);
        assert!(result.is_some());
        assert!(files.contains(&result.unwrap()));
    }

    #[test]
    fn test_sequential_ordering() {
        let files = vec![
            PathBuf::from("a.jpg"),
            PathBuf::from("b.jpg"),
            PathBuf::from("c.jpg"),
        ];
        let strategy = SequentialOrdering::new();

        let first = strategy.next(&files).unwrap();
        let second = strategy.next(&files).unwrap();
        let third = strategy.next(&files).unwrap();

        assert_ne!(first, second);
        assert_ne!(second, third);
    }
}
