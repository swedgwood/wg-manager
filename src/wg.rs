/// Minimal bindings to the `wg` binary.
use std::{
    io::{BufRead, Write},
    path::Path,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

/// Struct that represents a handle to the wg binary.
#[derive(Deserialize, Serialize)]
pub struct Wg {
    binary_path: String,
}

impl Wg {
    pub fn new(binary_path: String) -> Wg {
        Wg { binary_path }
    }

    pub fn genkey(&self) -> String {
        let output_bytes = Command::new(&self.binary_path)
            .arg("genkey")
            .output()
            .expect("`wg genkey` failed")
            .stdout;

        let privkey = strip_and_convert(&output_bytes);
        privkey
    }

    pub fn pubkey(&self, privkey: &String) -> String {
        let mut child = Command::new(&self.binary_path)
            .arg("pubkey")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("`wg pubkey` failed");

        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin.write_all(privkey.as_bytes()).unwrap();
        child_stdin.write_all(b"\n").unwrap();
        drop(child_stdin);

        let output_bytes = child.wait_with_output().unwrap().stdout;
        let pubkey = strip_and_convert(&output_bytes);
        pubkey
    }

    pub fn show_private_key(&self, interface: &str) -> String {
        todo!();
    }

    pub fn set_private_key(&self, interface: &str, path: &Path) {
        todo!();
    }

    pub fn show_listen_port(&self, interface: &str) -> u16 {
        todo!();
    }

    pub fn set_listen_port(&self, interface: &str, port: u16) {
        todo!();
    }

    pub fn show_peers(&self, interface: &str) -> Vec<String> {
        let output_bytes = Command::new(&self.binary_path)
            .arg("show")
            .arg(interface)
            .arg("peers")
            .output()
            .expect(&format!("`wg show {} peers` failed", interface))
            .stdout;

        let peers: Vec<String> = output_bytes
            .split(|x| *x == b'\n')
            .map(|b| std::str::from_utf8(b).unwrap().to_owned())
            .filter(|s| s.len() != 0)
            .collect();

        peers
    }

    pub fn show_allowed_ips(&self, interface: &str) -> Vec<Vec<String>> {
        let output_bytes = Command::new(&self.binary_path)
            .arg("show")
            .arg(interface)
            .arg("allowed-ips")
            .output()
            .expect(&format!("`wg show {} allowed-ips` failed", interface))
            .stdout;

        parse_table_strings(&output_bytes)
    }
}

fn strip_and_convert(bytes: &[u8]) -> String {
    let string_str = std::str::from_utf8(bytes).unwrap();
    let mut string = string_str.to_owned();
    if string.ends_with("\n") {
        string.pop();
    }
    string
}

/// Parses text table as a list of rows, which is a list of cells, where each cell is a byte string
fn parse_table(bytes: &[u8]) -> Vec<Vec<&[u8]>> {
    bytes
        .split(|b| *b == b'\n')
        .map(|row| {
            row.split(|b| b.is_ascii_whitespace())
                .filter(|bytes| !bytes.is_empty())
                .collect::<Vec<&[u8]>>()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

fn parse_table_strings(bytes: &[u8]) -> Vec<Vec<String>> {
    parse_table(bytes)
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| std::str::from_utf8(cell).unwrap().to_owned())
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table() {
        let bytes = b"\ncell11    cell12 cell13    \n   \n   cell21   cell22   cell23  \n  cell31 cell32 cell33\n\n";

        let parsed = parse_table(bytes);

        assert_eq!(
            parsed,
            vec![
                vec![b"cell11", b"cell12", b"cell13"],
                vec![b"cell21", b"cell22", b"cell23"],
                vec![b"cell31", b"cell32", b"cell33"]
            ]
        );
    }

    #[test]
    fn test_parse_table_strings() {
        let bytes = b"\ncell11    cell12 cell13    \n   \n   cell21   cell22   cell23  \n  cell31 cell32 cell33\n\n";

        let parsed = parse_table_strings(bytes);

        assert_eq!(
            parsed,
            vec![
                vec!["cell11", "cell12", "cell13"],
                vec!["cell21", "cell22", "cell23"],
                vec!["cell31", "cell32", "cell33"]
            ]
        );
    }
}
