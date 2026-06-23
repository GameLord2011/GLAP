use std::{
    fs,
    io::{Error, ErrorKind, stdin},
    path::Path,
};

struct HeaderChunk {
    flag: [u8; 4],
    length: u32,
    mode: u16,
    num_tracks: u16,
    tpq: u16,
}

struct Chunk {
    flag: [u8; 4],
    length: u32,
    data: Vec<u8>,
}

struct Message {
    delta: Vec<u8>, // Erm acktually the MIDI format uses Variable Length Quantities to represent the delta.
    status: u8,
    data: Vec<u8>
}

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
        return Err(Error::new(
            ErrorKind::InvalidFilename,
            "You can't point me to nothing, sorry!",
        ));
    }
    let p = Path::new(&path);
    let bytes = fs::read(p).unwrap();

    let header: HeaderChunk = HeaderChunk {
        flag: [bytes[0], bytes[1], bytes[2], bytes[3]],
        length: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        mode: u16::from_be_bytes([bytes[8], bytes[9]]),
        num_tracks: u16::from_be_bytes([bytes[10], bytes[11]]),
        tpq: u16::from_be_bytes([bytes[12], bytes[13]]),
    };

    if header.flag != [0x4D, 0x54, 0x68, 0x64] {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "The header flag is incorrect.",
        ));
    }

    let mut tracks: Vec<Chunk> = vec![];

    let mut next_chunk_offset: usize = (header.length + 8).try_into().unwrap();
    for _ in 0..header.num_tracks {
        println!("Offset: {}", next_chunk_offset);
        let l = u32::from_be_bytes([
            bytes[next_chunk_offset + 4],
            bytes[next_chunk_offset + 5],
            bytes[next_chunk_offset + 6],
            bytes[next_chunk_offset + 7],
        ]);
        tracks.insert(
            tracks.len(),
            Chunk {
                flag: [
                    bytes[next_chunk_offset],
                    bytes[next_chunk_offset + 1],
                    bytes[next_chunk_offset + 2],
                    bytes[next_chunk_offset + 3],
                ],
                length: l,
                data: bytes[(next_chunk_offset + 4)..(next_chunk_offset + 4 + l as usize)].to_vec(),
            },
        );
        next_chunk_offset += 8 + l as usize;
    }

    let mut valid_tracks = true;
    let mut first_inv_chunk = 0;

    for (i, j) in tracks.iter().enumerate() {
        if (j.flag != [0x4D, 0x54, 0x72, 0x6B]) || j.data.ends_with(&[0x00, 0xFF, 0x2F, 0x00]) {
            valid_tracks = false;
            first_inv_chunk = i;
            break;
        }
    }

    if valid_tracks == false {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Chunk {first_inv_chunk} has an invalid flag or no end command!")
        ))
    }

    Ok(())
}
