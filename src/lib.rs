use ftp::FtpStream;
use ftp_client::prelude::*;
use std::collections::HashSet;
use std::net::TcpStream;
use std::path::PathBuf;

pub fn fetch_directory_tree() {
    let mut conn = FtpStream::connect("144.92.217.20:21").unwrap();
    conn.login("anonymous", "anonymous@example.com").unwrap();

    let names = conn.nlst(Some("/pub/bmrb/timedomain")).unwrap();
    for directory in names {
        let entity_id = PathBuf::from(&directory)
            .file_name()
            .unwrap()
            .to_owned()
            .into_string()
            .unwrap();
        let out_path = PathBuf::from("./cache/timedomain_tree").join(format!("{}.json", entity_id));
        if out_path.exists() {
            continue;
        }

        let dir_contents = recursive_list_directory(&mut conn, &directory);
        let contents = serde_json::to_string_pretty(&dir_contents).unwrap();
        std::fs::create_dir_all("./cache/timedomain_tree").unwrap();
        std::fs::write(&out_path, &contents).unwrap();
    }
}

pub fn recursive_list_directory(conn: &mut FtpStream, path: &str) -> Vec<String> {
    let dir_files = conn.nlst(Some(path)).unwrap();
    dbg!(&dir_files);
    if dir_files.len() == 1 && dir_files[0] == path {
        return vec![dir_files[0].to_owned()];
    }

    let mut all_dir_files = vec![];
    for dir_file in dir_files {
        let mut subdir_files = recursive_list_directory(conn, &dir_file);
        all_dir_files.append(&mut subdir_files);
    }

    all_dir_files
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RemotePath(String);

impl Into<PathBuf> for RemotePath {
    fn into(self) -> PathBuf {
        PathBuf::from(self.0)
    }
}

/// Detect all directories containing raw NMR data.
pub fn detect_raw_directories(paths: &[RemotePath]) -> Vec<RemotePath> {
    let mut detected_directories = HashSet::new();
    for path in paths {
        if is_raw_nmr_file(&path) {
            let path = Into::<PathBuf>::into(path.clone());
            let parent_dir = path.parent().unwrap().to_string_lossy().to_owned();
            detected_directories.insert(RemotePath(parent_dir.to_string()));
        }
    }

    detected_directories.into_iter().collect()
}

/// Check whether the path can be identified as a file belonging to a raw NMR measurement.
pub fn is_raw_nmr_file(path: &RemotePath) -> bool {
    let KNOWN_BRUKER_FILENAMES = &["acqu"];
    for known_filename in KNOWN_BRUKER_FILENAMES {
        if Into::<PathBuf>::into(path.clone())
            .file_name()
            .unwrap()
            .to_string_lossy()
            == *known_filename
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
