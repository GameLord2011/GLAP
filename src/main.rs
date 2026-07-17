use std::{
    io::{Error, ErrorKind, stdin},
    path::Path,
};

use creak::{Decoder, SampleIterator};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

fn main() {
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
    }

    let samples = Decoder::open(Path::new(&path)).unwrap();
    let info = samples.info();

    println!("{}; {}; {}", info.channels(), info.format(), info.sample_rate());

    let mut n = 0;
    'sampler: for i in samples.into_samples().unwrap() {
        n += 1;
        let sample = i.unwrap();
        println!("{sample}");
        if n == 100000 {
            break 'sampler;
        }
    }
}
