use std::{fs::File, io::Read};

use clap::Parser;

/// Compare binary files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    file1: String,

    #[arg()]
    file2: String,
}

const BUFFER_SIZE: usize = 1024;

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let mut f1 = File::open(args.file1)?;
    let mut f2 = File::open(args.file2)?;

    let mut buffer1 = [0u8; BUFFER_SIZE];
    let mut buffer2 = [0u8; BUFFER_SIZE];

    let mut offset = 0;

    loop {
        let n1 = f1.read(&mut buffer1)?;
        let n2 = f2.read(&mut buffer2)?;

        let n = std::cmp::min(n1, n2);

        if n1 != n2 {
            eprintln!("one of the files ended prematurely (n1 {} n2 {})", n1, n2);
        }

        // EOF with empty buffer
        if n == 0 {
            break;
        }

        compare_buffers(&buffer1[..n], &buffer2[..n], offset);

        // EOF
        if n < BUFFER_SIZE {
            break;
        }

        offset += BUFFER_SIZE;
    }

    Ok(())
}

fn compare_buffers(buffer1: &[u8], buffer2: &[u8], buffer_offset: usize) {
    for i in 0..buffer1.len() {
        let v1 = buffer1[i];
        let v2 = buffer2[i];
        let offset = buffer_offset + i;
        if v1 != v2 {
            println!("Offset {} file1 {} file2 {}", offset, v1, v2);
        }
    }
}
