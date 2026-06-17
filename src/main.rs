use std::{io::{stdin, ErrorKind}, path::Path, fs};

fn main() -> std::io::Result<()> {
    println!("Whar is the file:");
    let mut path = String::new();
    stdin().read_line(&mut path).unwrap();
    if let Some('\n') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\r') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\"') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\"') = path.chars().rev().next_back() {
        path.remove(0);
    }
    if let Some('\'') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\'') = path.chars().rev().next_back() {
        path.remove(0);
    }

    if path.is_empty() {
        println!("You can't point me to nothing, sorry!");
    } else {
        let p = Path::new(&path);
        let bytes = fs::read(p).unwrap();

        if  !bytes.starts_with(&[0x4d, 0x54, 0x68, 0x64]) &&
            !bytes.ends_with(&[0x00, 0x00, 0xFF, 0x2F, 0x00]) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "This file is corrupted, incomplete, and/or the filetype is unsupported.",
            ));
        }

        let len: u32 = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let mode: u16 = u16::from_be_bytes([bytes[8], bytes[9]]);
        let num_tracks: u16 = u16::from_be_bytes([bytes[10], bytes[11]]);
        let tpq: u16 = u16::from_be_bytes([bytes[12], bytes[13]]);

        // This at least works for the midi I'm working with, will extend my testing set in the near future.
        let name_len = bytes[25] as usize;
        let name = String::from_utf8(bytes[26..(26 + name_len)].to_vec()).unwrap();
        println!("The name of the song is: {}", name);
    }

    Ok(())
}
