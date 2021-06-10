use std::{
    ascii::AsciiExt,
    io::{BufRead, Write},
    process::{Command, Stdio},
};

pub fn wg_genkey() -> String {
    let output_bytes = Command::new("wg")
        .arg("genkey")
        .output()
        .expect("`wg genkey` failed")
        .stdout;

    let privkey = strip_and_convert(&output_bytes);
    privkey
}

pub fn wg_pubkey(privkey: &String) -> String {
    let mut child = Command::new("wg")
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

pub fn wg_show_peers(interface: &str) -> Vec<String> {
    let output_bytes = Command::new("wg")
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

fn strip_and_convert(bytes: &[u8]) -> String {
    let string_str = std::str::from_utf8(bytes).unwrap();
    let mut string = string_str.to_owned();
    if string.ends_with("\n") {
        string.pop();
    }
    string
}
