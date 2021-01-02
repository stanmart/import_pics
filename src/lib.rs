use chrono::{DateTime, Utc};
use fs::DirEntry;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time;

#[derive(Debug)]
pub struct AnalyzedFile {
    file: fs::DirEntry,
    stem: String,
    ext: String,
    raw_creation_time: Option<time::SystemTime>,
    creation_time: Option<DateTime<Utc>>,
    to_copy: Option<bool>,
}

impl AnalyzedFile {
    pub fn from_direntry(file: fs::DirEntry) -> Self {
        let path = file.path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or("".to_string());
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string().to_lowercase())
            .unwrap_or("".to_string());

        let raw_creation_time = file.metadata().ok().and_then(|md| md.created().ok());
        let creation_time: Option<DateTime<Utc>> = raw_creation_time // TODO: time zones?
            .map(|ct| DateTime::from(ct));

        AnalyzedFile {
            file,
            stem,
            ext,
            raw_creation_time,
            creation_time,
            to_copy: None,
        }
    }

    pub fn dest_subdir_name(&self) -> String {
        match self.creation_time {
            Some(dt) => dt.format("%Y-%m-%d").to_string(),
            None => "undated".to_string(),
        }
    }
}

pub fn list_dir(dir: PathBuf, recursive: bool) -> io::Result<Vec<DirEntry>> {
    let entries = fs::read_dir(dir)?.filter_map(Result::ok);

    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in entries {
        if let Ok(ft) = entry.file_type() {
            if ft.is_file() {
                files.push(entry)
            } else if ft.is_dir() {
                dirs.push(entry)
            }
        }
    }

    if recursive {
        for dir in dirs {
            if let Ok(mut new_files) = list_dir(dir.path(), recursive) {
                files.append(&mut new_files);
            }
        }
    }

    Ok(files)
}

pub fn analyze_files(files: Vec<DirEntry>, extensions: &Vec<&str>) -> Vec<AnalyzedFile> {
    files
        .into_iter()
        .map(AnalyzedFile::from_direntry)
        .filter(|af| extensions.contains(&af.ext.as_str()))
        .collect()
}
