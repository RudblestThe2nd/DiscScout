use jwalk::WalkDir;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub _mtime: u64,
}

pub fn scan(root: &str) -> Vec<FileInfo> {
    let entries: Vec<_> = WalkDir::new(root)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    entries
        .into_par_iter()
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            Some(FileInfo {
                path: entry.path().to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                size: if meta.is_file() { meta.len() } else { 0 },
                is_dir: meta.is_dir(),
                _mtime: meta
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs(),
            })
        })
        .collect()
}
