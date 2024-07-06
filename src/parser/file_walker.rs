use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::io;

use log_derive::{logfn, logfn_inputs};

static DROP_FILE_EXTENSION: &str = ".drop";

pub struct FileWalker {}

impl FileWalker {
    #[logfn(
        ok = "TRACE",
        err = "ERROR",
        fmt = "Resolved drop files: {:?}",
        log_ts = true
    )]
    pub fn resolve_drop_files(dir: &str) -> Result<Vec<PathBuf>, io::Error> {
        let path = Path::new(&dir);
        let abs = path.canonicalize();
        let mut drop_files: Vec<PathBuf> = Vec::default();

        let _ = FileWalker::walk(&abs.unwrap(), &mut drop_files);

        Ok(drop_files)
    }

    #[logfn(err = "ERROR")]
    #[logfn_inputs(TRACE, fmt = "walking {:?} {:?}")]
    fn walk(root_path: &PathBuf, drop_files_discovered: &mut Vec<PathBuf>) -> io::Result<()> {
        for file in read_dir(root_path)? {
            let file = file?;

            let pathbuf = file.path();

            if pathbuf.is_dir() {
                let cur_path = pathbuf.as_path().to_str().unwrap();

                if cur_path.contains("__test__")
                    || cur_path.contains("/target")
                    || cur_path.contains(".git")
                    || cur_path.contains("/src")
                {
                    continue;
                }

                FileWalker::walk(&pathbuf, drop_files_discovered)?;
            } else if pathbuf.is_file() {
                let file_name = pathbuf.file_name().and_then(OsStr::to_str).unwrap();

                let is_dropfile = file_name.ends_with(DROP_FILE_EXTENSION);

                if is_dropfile {
                    drop_files_discovered.push(pathbuf.clone());
                }
            }
        }

        Ok(())
    }
}
