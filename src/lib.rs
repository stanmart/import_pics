use std::fs;
use std::time;
use std::io;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

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
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or("".to_string());
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string().to_lowercase())
            .unwrap_or("".to_string());

        let raw_creation_time = file
            .metadata().ok()
            .and_then(|md| md.created().ok());
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
}

pub fn analyze_dir(dir: PathBuf, extensions: Option<Vec<&str>>, recursive: bool) -> io::Result<Vec<AnalyzedFile>> {
    let extensions = match extensions {
        Some(ext) => ext,
        None => vec!["jpg", "jpeg", "tif", "tiff", "raw", "arw", "mp4"]
    };

    let entries = fs::read_dir(dir)?
        .filter_map(Result::ok);

    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in entries {
        match entry.file_type() {
            Ok(ft) => {
                if ft.is_file() {
                    files.push(entry) // What to do about symlinks?
                } else if ft.is_dir() {
                    dirs.push(entry)
                }
            },
            Err(_) => continue, // Should there be some message here?
        }
    }
    
    let mut analyzed_files: Vec<AnalyzedFile> = files.into_iter()
            .filter(|f| match f.file_type() {
                Ok(ft) => ft.is_file(),  // TODO: what to do about symlinks?
                Err(_) => false
            })
            .map(AnalyzedFile::from_direntry)
            .filter(|af| extensions.contains(&af.ext.as_str()))
            .collect();

    if recursive {
        for analyzed_file in dirs.into_iter() 
            .flat_map(|dir| match analyze_dir(dir.path(), Some(extensions.clone()), recursive) {
                Ok(sub_files) => sub_files,
                Err(_) => vec![],
            }) {
                analyzed_files.push(analyzed_file)
            }
    }
        
    Ok(analyzed_files)

}
