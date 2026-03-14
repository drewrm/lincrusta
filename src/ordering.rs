use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait ImageOrdering: Send + Sync {
    fn next(&self, files: &[std::path::PathBuf]) -> Option<std::path::PathBuf>;
}

pub struct RandomOrdering;

impl ImageOrdering for RandomOrdering {
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

impl ImageOrdering for SequentialOrdering {
    fn next(&self, files: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
        if files.is_empty() {
            return None;
        }

        let index = self.index.fetch_add(1, Ordering::Relaxed) % files.len();
        Some(files[index].clone())
    }
}

pub fn get_image_files(dir: &Path) -> Option<Vec<std::path::PathBuf>> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
                )
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

pub fn get_next_image(dir: &Path, strategy: &dyn ImageOrdering) -> Option<String> {
    let files = get_image_files(dir)?;
    strategy
        .next(&files)
        .map(|p| p.to_string_lossy().to_string())
}
