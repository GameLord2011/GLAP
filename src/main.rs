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

        if !bytes.starts_with(&[0x4d, 0x54, 0x68, 0x64]) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "This file is corrupted or the filetype is unsupported.",
            ));
        }

        
    }

    Ok(())
}
