use chrono::{DateTime, Utc};
use custom_error::custom_error;
use fs::DirEntry;
use indicatif::ProgressBar;
use std::time;
use std::{collections::HashMap, fs};
use std::{io, path::Path, path::PathBuf};

custom_error! {pub CopyError
    Io {
        source: io::Error,
        path: PathBuf
    } = @{format!("{path}: {source}", source=source, path=path.display())},
}

#[derive(Debug)]
pub struct FileWithMetadata {
    file: fs::DirEntry,
    name: String,
    ext: String,
    raw_creation_time: Option<time::SystemTime>,
    creation_time: Option<DateTime<Utc>>,
}

impl FileWithMetadata {
    pub fn from_direntry(file: fs::DirEntry) -> Self {
        let path = file.path();
        let name = path
            .file_name()
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

        FileWithMetadata {
            file,
            name,
            ext,
            raw_creation_time,
            creation_time,
        }
    }

    pub fn dest_subdir_name(&self) -> String {
        match self.creation_time {
            Some(dt) => dt.format("%Y-%m-%d").to_string(),
            None => "undated".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ProcessedFile {
    New(FileWithMetadata),
    Existing(FileWithMetadata),
}

impl ProcessedFile {
    pub fn from_file(file: FileWithMetadata, target_dir: &Path) -> Self {
        let path = target_dir.join(file.dest_subdir_name()).join(&file.name);
        match path.exists() {
            true => ProcessedFile::Existing(file),
            false => ProcessedFile::New(file),
        }
    }
}

fn list_dir(dir: &Path, recursive: bool) -> io::Result<Vec<DirEntry>> {
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
            if let Ok(mut new_files) = list_dir(&dir.path(), recursive) {
                files.append(&mut new_files);
            }
        }
    }

    Ok(files)
}

pub fn analyze_source_dir(
    dir: &Path,
    recursive: bool,
    extensions: &Vec<&str>,
) -> io::Result<Vec<FileWithMetadata>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Analyzing source directory");
    pb.enable_steady_tick(50);
    let files: Vec<FileWithMetadata> = list_dir(dir, recursive)?
        .into_iter()
        .map(FileWithMetadata::from_direntry)
        .filter(|f| extensions.contains(&f.ext.as_str()))
        .collect();
    let end_message = format!("Found {} files", files.len());
    pb.finish_with_message(&end_message);
    Ok(files)
}

pub fn group_files(
    files: Vec<FileWithMetadata>,
    target_dir: &Path,
) -> HashMap<String, Vec<ProcessedFile>> {
    let mut file_map: HashMap<String, Vec<ProcessedFile>> = HashMap::new();
    for file in files {
        let subfolder_name = file.dest_subdir_name();
        let processed_file = ProcessedFile::from_file(file, target_dir);
        let file_vec = file_map.entry(subfolder_name).or_insert_with(|| vec![]);
        file_vec.push(processed_file);
    }

    file_map
}

pub fn copy_files(
    grouped_files: HashMap<String, Vec<ProcessedFile>>,
    target_dir: &Path,
    num_new_files: Option<u64>,
) -> Vec<Result<u64, CopyError>> {
    let pb = match num_new_files {
        Some(num) => ProgressBar::new(num),
        None => ProgressBar::new_spinner(),
    };
    pb.set_message("Copying files");

    let mut copy_results = Vec::new();
    for (subdir, files) in grouped_files {
        let subdir_path = target_dir.join(&subdir);
        let skip = if subdir_path.is_dir() {
            false
        } else {
            match fs::create_dir(&subdir_path) {
                Ok(_) => false,
                Err(e) => {
                    pb.println(format!("Could not create directory {}", &subdir));
                    copy_results.push(Err(CopyError::Io {
                        source: e,
                        path: subdir_path.to_owned(),
                    }));
                    true
                }
            }
        };
        for processed_file in files {
            if let ProcessedFile::New(file) = processed_file {
                if !skip {
                    let target_path = subdir_path.join(file.name);
                    match fs::copy(&file.file.path(), &target_path) {
                        Ok(size) => copy_results.push(Ok(size)),
                        Err(e) => copy_results.push(Err(CopyError::Io {
                            source: e,
                            path: subdir_path.to_owned(),
                        })),
                    };
                }
                pb.inc(1);
            }
        }
    }
    pb.finish_with_message("Finished");
    copy_results
}
