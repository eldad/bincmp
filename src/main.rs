use std::{fs::File, io::{Read, Write, stdout}};

use clap::{Parser, ValueEnum};
use tabwriter::TabWriter;

#[derive(ValueEnum, Clone, Debug)]
enum ValueOutputFormat {
    Hex,
    Decimal,
    Both,
}

/// Compare binary files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    file1: String,

    #[arg()]
    file2: String,

    #[arg(short, long, default_value = "both")]
    format: ValueOutputFormat,
}

const BUFFER_SIZE: usize = 1024;

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let mut f1 = File::open(&args.file1)?;
    let mut f2 = File::open(&args.file2)?;

    let mut buffer1 = [0u8; BUFFER_SIZE];
    let mut buffer2 = [0u8; BUFFER_SIZE];

    let mut offset = 0;
    let mut tw = TabWriter::new(stdout()).padding(5).alignment(tabwriter::Alignment::Right);

    match &args.format {
        ValueOutputFormat::Hex | ValueOutputFormat::Decimal => writeln!(tw, "OFFSET\tFILE1\tFILE2\t")?,
        ValueOutputFormat::Both => writeln!(tw, "OFFSET\tHex\tFILE1\tHex\tFILE2\tHex\t")?,
    }

    loop {
        let n1 = f1.read(&mut buffer1)?;
        let n2 = f2.read(&mut buffer2)?;

        let n = std::cmp::min(n1, n2);

        if n != 0 {
            compare_buffers(&mut tw,&buffer1[..n], &buffer2[..n], offset, &args.format)?;
        }

        // EOF
        if n < BUFFER_SIZE {
            match n1.cmp(&n2) {
                std::cmp::Ordering::Less => eprintln!("NOTE: The second file ({}) is larger than the first file ({}).", args.file2, args.file1),
                std::cmp::Ordering::Greater => eprintln!("NOTE: The first file ({}) is larger than the second file ({}).", args.file1, args.file2),
                std::cmp::Ordering::Equal => (),
            };
            break;
        }

        offset += BUFFER_SIZE;
    }

    tw.flush()?;

    Ok(())
}

fn compare_buffers<T: Write>(w: &mut T, buffer1: &[u8], buffer2: &[u8], buffer_offset: usize, format: &ValueOutputFormat) -> eyre::Result<()> {
    for i in 0..buffer1.len() {
        let v1 = buffer1[i];
        let v2 = buffer2[i];
        let offset = buffer_offset + i;
        if v1 != v2 {
            match format {
                ValueOutputFormat::Hex => writeln!(w, "{:x}\t{:x}\t{:x}\t", offset, v1, v2)?,
                ValueOutputFormat::Decimal => writeln!(w, "{}\t{}\t{}\t", offset, v1, v2)?,
                ValueOutputFormat::Both => writeln!(w, "{}\t{:x}\t{}\t{:x}\t{}\t{:x}\t", offset, offset, v1, v1, v2, v2)?,
            }
        }
    }
    Ok(())
}
