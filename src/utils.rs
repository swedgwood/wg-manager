use ipnet::Ipv4Net;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

pub fn serialize_ipv4net<S>(ipv4net: &Ipv4Net, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ipv4net.to_string().serialize(serializer)
}

pub fn deserialize_ipv4net<'de, D>(deserializer: D) -> Result<Ipv4Net, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|x| x.parse::<Ipv4Net>().map_err(|e| D::Error::custom(e)))
}

// Takes in a table of strings (vec of rows, each row is a vec of strings)
// Returns a vec of print lines
pub fn cli_table(table: Vec<Vec<&str>>) -> Vec<String> {
    let mut column_widths: Vec<usize> = Vec::new();

    for row in table.iter() {
        for (i, cell) in row.iter().enumerate() {
            let width = cell.len();
            if i >= column_widths.len() {
                column_widths.push(width);
            } else if column_widths[i] < width {
                column_widths[i] = width;
            }
        }
    }

    let mut lines: Vec<String> = Vec::new();

    for row in table.iter() {
        let mut line = String::new();

        for (i, cell) in row.iter().enumerate() {
            let max_width = column_widths[i];
            let padding = max_width - cell.len();
            line.push_str(&" ".repeat(padding));
            line.push_str(cell);
            line.push(' '); // for spacing
        }
        line.pop(); // remove the last extra space

        lines.push(line);
    }

    lines
}

/// Generates a lock path to specific file path.
///
/// Same file path will always produce same lock path.
/// Function assumes `file_path.file_name()` returns a `Some`, i.e. that the file path has a file name.
pub fn lock_path(file_path: &Path) -> PathBuf {
    let file_name = file_path.file_name().unwrap();

    let mut lock_name = OsString::from(".");
    lock_name.push(file_name);
    lock_name.push(".lck");

    file_path.with_file_name(lock_name)
}

/// An error with file locking
#[derive(Debug)]
pub enum LockError {
    MalformedLockExists,
    LockExists(u32),
    IOError(std::io::Error),
}

impl ToString for LockError {
    fn to_string(&self) -> String {
        match &self {
            LockError::MalformedLockExists => "lock exists, but it is malformed".into(),
            LockError::LockExists(e) => format!("lock exists with process id: {}", e),
            LockError::IOError(e) => e.to_string(),
        }
    }
}

impl From<std::io::Error> for LockError {
    fn from(err: std::io::Error) -> Self {
        LockError::IOError(err)
    }
}

/// Simple file-based lock for the config file
///
/// Lock acquisition rules
/// 1. if path is not a file, fail
/// 2. if path has incorrect permissions, fail
/// 3. if path is a file and exists:
///   a. and contains a valid number (process id), fail with that id
///   b. and no valid number, fail
///
/// Lock dropping rules
/// 1. Releasing a lock cannot fail in the sense that an Err is returned, so will require
///    manual intervention if the lock cannot be deleted.
#[derive(Debug)]
pub struct Lock(PathBuf);

impl Lock {
    /// Acquire a lock in the form of a file, stored at `lock_path`
    ///
    /// Lock is released through `.drop()` (provided by `Drop` trait)
    pub fn acquire(lock_path: impl Into<PathBuf>) -> Result<Self, LockError> {
        let lock_path = lock_path.into();

        if lock_path.exists() {
            let locking_process_id: u32 = std::fs::read_to_string(lock_path)?
                .parse()
                .or(Err(LockError::MalformedLockExists))?;

            Err(LockError::LockExists(locking_process_id))
        } else {
            let process_id = std::process::id();

            std::fs::write(&lock_path, process_id.to_string().as_bytes())?;
            Ok(Self(lock_path))
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        // TODO: We currently ignore if the lock isn't successfully deleted, maybe not good behaviour?
        let _ = std::fs::remove_file(&self.0);
    }
}
