/*
 * bincmp: analyze difference between binaries
 * Copyright (C) 2023 Eldad Zack
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    fs::File,
    io::{stdout, Read, Write},
};

use clap::{Parser, ValueEnum};
use tabwriter::TabWriter;

#[derive(ValueEnum, Clone, Debug)]
enum ValueOutputFormat {
    Hex,
    Decimal,
    Binary,
    Combined,
}

/// Compare binary files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    file1: String,

    #[arg()]
    file2: String,

    #[arg(short, long, default_value = "hex")]
    format: ValueOutputFormat,

    #[arg(short, long)]
    /// Search only for a single bit flip
    single_bitflip_only: bool,
}

const BUFFER_SIZE: usize = 1024;

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let mut f1 = File::open(&args.file1)?;
    let mut f2 = File::open(&args.file2)?;

    let mut buffer1 = [0u8; BUFFER_SIZE];
    let mut buffer2 = [0u8; BUFFER_SIZE];

    let mut offset = 0;
    let mut tw = TabWriter::new(stdout())
        .padding(5)
        .alignment(tabwriter::Alignment::Right);

    match &args.format {
        ValueOutputFormat::Combined => writeln!(tw, "OFFSET\tHex\tFILE1\tHex\tFILE2\tHex\t")?,
        _ => writeln!(tw, "OFFSET\tFILE1\tFILE2\t")?,
    }

    loop {
        let n1 = f1.read(&mut buffer1)?;
        let n2 = f2.read(&mut buffer2)?;

        let n = std::cmp::min(n1, n2);

        if n != 0 {
            compare_buffers(
                &mut tw,
                &buffer1[..n],
                &buffer2[..n],
                offset,
                &args.format,
                args.single_bitflip_only,
            )?;
        }

        // EOF
        if n < BUFFER_SIZE {
            match n1.cmp(&n2) {
                std::cmp::Ordering::Less => eprintln!(
                    "NOTE: The second file ({}) is larger than the first file ({}).",
                    args.file2, args.file1
                ),
                std::cmp::Ordering::Greater => eprintln!(
                    "NOTE: The first file ({}) is larger than the second file ({}).",
                    args.file1, args.file2
                ),
                std::cmp::Ordering::Equal => (),
            };
            break;
        }

        offset += BUFFER_SIZE;
    }

    tw.flush()?;

    Ok(())
}

fn compare_buffers<T: Write>(
    w: &mut T,
    buffer1: &[u8],
    buffer2: &[u8],
    buffer_offset: usize,
    format: &ValueOutputFormat,
    bitflip_only: bool,
) -> eyre::Result<()> {
    for i in 0..buffer1.len() {
        let v1 = buffer1[i];
        let v2 = buffer2[i];
        let offset = buffer_offset + i;

        let is_diff = bitflip_only && is_bitflipped(v1, v2) || !bitflip_only && (v1 != v2);
        if is_diff {
            match format {
                ValueOutputFormat::Binary => writeln!(w, "{:x}\t{:08b}\t{:08b}\t", offset, v1, v2)?,
                ValueOutputFormat::Hex => writeln!(w, "{:x}\t{:x}\t{:x}\t", offset, v1, v2)?,
                ValueOutputFormat::Decimal => writeln!(w, "{}\t{}\t{}\t", offset, v1, v2)?,
                ValueOutputFormat::Combined => writeln!(
                    w,
                    "{}\t{:x}\t{}\t{:x}\t{}\t{:x}\t",
                    offset, offset, v1, v1, v2, v2
                )?,
            }
        }
    }
    Ok(())
}

fn is_bitflipped(v1: u8, v2: u8) -> bool {
    let v = v1 ^ v2;
    if v == 0 {
        return false;
    }
    v & (v - 1) == 0
}
