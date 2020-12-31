use std::fs;
use std::time;
use std::io;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct AnalyzedFile {
    file: fs::DirEntry,
    stem: Option<String>,
    ext: Option<String>,
    raw_creation_time: Option<time::SystemTime>,
    creation_time: Option<DateTime<Utc>>,
}

impl AnalyzedFile {
    pub fn from_direntry(file: fs::DirEntry) -> Self {

        let path = file.path();
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        let raw_creation_time = file
            .metadata().ok()
            .and_then(|md| md.created().ok());
        let creation_time: Option<DateTime<Utc>> = raw_creation_time
            .map(|ct| DateTime::from(ct));

        AnalyzedFile {
            file,
            stem,
            ext,
            raw_creation_time,
            creation_time,
        }
    }
}
