use std::{
    fs,
    io::{Error, ErrorKind, stdin},
    path::Path,
};

/**
    The header track.
 */
struct HeaderTrack {
    flag: [u8; 4],
    length: u32,
    mode: u16,
    num_tracks: u16,
    tpq: u16,
}

/**
    A generic track.
 */
struct Track {
    flag: [u8; 4],
    length: u32,
    data: Vec<u8>,
}

/**
    A MIDI message. 
 */
#[derive(Clone)]
struct Message {
    delta: u32, // Erm acktually the MIDI format uses Variable Length Quantities to represent the delta.
    channel: Option<u8>,
    status: u8,
    data: Vec<u8>,
}

impl Message {
    /**
        Creates a new (empty) message and returns it.
     */
    fn new() -> Self {
        Self {
            delta: 0,
            channel: None,
            status: 0,
            data: vec![]
        }
    }
}

#[derive(Clone, Copy)]
struct Vlq {
    value: u32,
    length: usize
}

impl Vlq {
    fn from_bytes(possible_bytes: &[u8]) -> Self {
        let mut val = 0;
        let mut len = 0;

        for byte in possible_bytes {
            let actual_byte = (byte & 0b01111111) as u32;
            val = (val << 7) | actual_byte;
            len += 1;

            if byte & 0b10000000 == 0b00000000 {
                break;
            }
        }

        Self {
            value: val,
            length: len
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

fn main() -> std::io::Result<()> {
    let mut path = String::new();
    let p = std::env::args().nth(1);
    match p {
        Some(n) => path = n,
        None => {
            println!("Whar is the file:");
            stdin().read_line(&mut path).unwrap();
        }
    }
    path = path.trim_matches(&['\r', '\n', '"', '\'']).to_owned();

    if path.is_empty() {
        println!("You can't point me to nothing, sorry!");
        return Err(Error::new(
            ErrorKind::InvalidFilename,
            "You can't point me to nothing, sorry!",
        ));
    }
    let bytes = fs::read(Path::new(&path)).unwrap();

    let header: HeaderTrack = HeaderTrack {
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

    let mut tracks: Vec<Track> = vec![];

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
            Track {
                flag: [
                    bytes[next_chunk_offset],
                    bytes[next_chunk_offset + 1],
                    bytes[next_chunk_offset + 2],
                    bytes[next_chunk_offset + 3],
                ],
                length: l,
                data: bytes[(next_chunk_offset + 8)..(next_chunk_offset + 8 + l as usize)].to_vec(),
            },
        );
        next_chunk_offset += 8 + l as usize;
    }

    let mut valid_tracks = true;
    let mut first_inv_chunk = 0;

    for (i, j) in tracks.iter().enumerate() {
        if (j.flag != [0x4D, 0x54, 0x72, 0x6B]) || !j.data.ends_with(&[0x00, 0xFF, 0x2F, 0x00]) {
            valid_tracks = false;
            first_inv_chunk = i;
            break;
        }
    }

    if valid_tracks == false {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Chunk {first_inv_chunk} has an invalid flag or no end command!"),
        ));
    }

    // let mut deltas: Vec<u8> = vec![];

    for (i, track) in tracks.iter().enumerate() {
        println!("\nNew Track; idx {i}");
        let d = track.data.to_vec();
        let mut offset: usize = 0;
        let mut messages: Vec<Message> = vec![];
        loop {
            let mut message = Message::new();
            println!("Offset = {}", offset);
            let vlq = Vlq::from_bytes(&[d[offset], d[offset + 1], d[offset + 2], d[offset + 3]]);

            println!("D = {}", vlq.value);

            let status = d[offset + vlq.length];

            match status {
                0x80..=0x8F /* Note off */ => {
                    offset += vlq.length + 3;
                    // messages.insert(messages.len())
                },
                0x90..=0x9F /* Note on */ => {
                    offset += vlq.length + 3;
                },
                0xA0..=0xAF /* Polyphonic after touch (what) */ => {
                    offset += vlq.length + 3;
                },
                0xB0..=0xBF /* Control change */ => {
                    match d[offset + vlq.length + 1] {
                        0x0 => offset += vlq.length + 6,
                        _ => {}
                    }
                },
                0xC0..=0xCF /* Program change */ => {
                    offset += vlq.length + 2;
                },
                0xD0..=0xDF /* Channel After Touch (wat) */ => {
                    offset += vlq.length + 2;
                },
                0xE0..=0xEF /* Pitch Wheel */ => {
                    offset += vlq.length + 4;
                },
                0xF0 /* SysEx; this uses vlq's whyyyyyyyyy */ => {
                    offset += vlq.length
                },
                0xF1 /* Time Code Qtr Frame (??) */ => {
                    offset += vlq.length;
                },
                0xF2 /* Song Position Pointer (Sounds important.) */ => {
                    offset += vlq.length + 3;
                },
                0xF3 /* Song Select */ => {
                    offset += vlq.length
                },
                0xF4..=0xF5 /* Undefined (why tho?) */ => {},
                0xF6 /* Tune Request */ => {
                    offset += vlq.length + 2;
                },
                0xF7 /* EndOfSysEx (wait what now) */ => {
                    offset += vlq.length;
                },
                0xF8 /* Timing Clock (makes sense) */ => {
                    offset += vlq.length;
                },
                0xF9 /* Undefined */ => {
                    offset += vlq.length;
                },
                0xFA /* Start */ => {
                    offset += vlq.length;
                },
                0xFB /* Continue */ => {
                    offset += vlq.length;
                },
                0xFC /* Stop */ => {
                    offset += vlq.length;
                },
                0xFF /* Meta Event */ => {
                    println!("Meta event {:X}.", d[offset + vlq.length + 1]);
                    let meta_len = d[offset + vlq.length + 2] as usize + vlq.length + 3;
                    match d[offset + vlq.length + 1] {
                        0x00 /* Sequence number */ => {},
                        0x01..=0x07 /* Things that use meta lengths */ => {
                            println!("String event");
                            offset += meta_len;
                        },
                        0x20 /* Channel Prefix */ => {},
                        0x2F /* End of track */ => {
                            println!("Eot; {}, {:X}", status, d[offset + vlq.length + 1]);
                            break;
                        },
                        0x51 /* Set Tempo */ => {
                            offset += meta_len;
                        },
                        0x54 /* SMTPE Offset */ => {},
                        0x58 /* Time Signature */ => {
                            offset += meta_len;
                        },
                        0x59 /* Key Signature */ => {
                            offset += meta_len;
                        },
                        0x7F /* Sequencer Specific */ => {},
                        _ => {
                            println!("Unknown meta event; type is {:X}; length is {} bytes", d[offset + vlq.length + 1], meta_len);
                            break;
                        }
                    }
                },
                _ => {
                    println!("Unknown status {:X}", status);
                    break;
                }
            }
        }
    }

    Ok(())
}
