#![allow(unused)]

use clap::Parser;
use crc::crc32;
use std::fs::File;
use std::io::prelude::*;

mod crc;

/// A utility to help generate uf2 files which can be flashed to an
/// rp2040 microcontroller.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The bootrom binary file that you want to flash to the pico
    /// which should not exceed 252 bytes.
    #[arg(short, long)]
    bootrom: String,

    /// The program binary file that you want to flash to the pico
    /// which will be placed in memory 256-byte aligned.
    #[arg(short, long)]
    progdata: String,

    /// The output file name
    #[arg(short, long)]
    output: String,
}

#[derive(Clone)]
struct Uf2Block {
    magic_0: u32,
    magic_1: u32,
    flags: u32,
    target_addr: u32,
    payload_size: u32,
    block_no: u32,
    num_blocks: u32,
    file_size: u32,
    data: Vec<u8>,
    magic_end: u32,
}

impl Uf2Block {
    pub fn allocate(target_addr: u32, block_no: u32, num_blocks: u32, data: Vec<u8>) -> Self {
        return Uf2Block {
            magic_0: 0x0A324655,
            magic_1: 0x9E5D5157,
            flags: 0x00002000, // familyID present
            // 0x10000000 - Flash
            // 0x20000000 - Main RAM
            target_addr: target_addr,
            payload_size: 256, // Per the spec, this is apparently non-negotiable
            block_no: block_no,
            num_blocks: num_blocks,
            file_size: 0xe48bff56, // Family ID for RP2040
            data: data,
            magic_end: 0x0AB16F30,
        };
    }
}

struct Uf2 {
    blocks: Vec<Uf2Block>,
}

fn write_little_endian(vec: &mut Vec<u8>, block: u32) {
    vec.push((block & 0xFF) as u8);
    vec.push(((block & 0xFF00) >> 8) as u8);
    vec.push(((block & 0xFF0000) >> 16) as u8);
    vec.push((block >> 24) as u8);
}

impl Uf2 {
    pub fn create(hex_file: &[u8]) -> Self {
        let mut blocks = Vec::new();

        // The first chunk is special
        let mut first_chunk = hex_file.take(252);
        let remaining_bytes = hex_file.iter().skip(256).collect::<Vec<&u8>>();
        let chunks = remaining_bytes.chunks(256);
        let num_blocks = chunks.len() as u32;
        let base_addr = 0x10000000;

        // First chunk is magical and must have a crc
        let mut buffer = Vec::new();
        first_chunk.read_to_end(&mut buffer);
        let remaining = 252 - buffer.len();
        // Must have 252 bytes
        for _ in 0..remaining {
            buffer.push(0);
        }
        let crc = crc32(buffer.as_slice());

        // Add the crc as the last 4 bytes in little endian
        write_little_endian(&mut buffer, crc);
        for _ in buffer.len()..476 {
            buffer.push(0);
        }
        blocks.push(Uf2Block::allocate(base_addr, 0, num_blocks + 1, buffer));

        // For each chunk, create
        for chunk in chunks {
            let mut data = chunk.to_vec().iter().map(|x| (**x)).collect::<Vec<u8>>();
            for _ in data.len()..476 {
                data.push(0);
            }

            blocks.push(Uf2Block::allocate(
                base_addr + blocks.len() as u32 * 256,
                blocks.len() as u32,
                num_blocks + 1,
                data,
            ));
        }

        let block_count = blocks.len();
        println!("{block_count} blocks generated");

        return Uf2 { blocks: blocks };
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let bytes = self.blocks.len() * 512;
        let mut buf: Vec<u8> = Vec::with_capacity(bytes);

        // For each chunk, write the bytes
        // And remember it's all little endian
        self.blocks.iter().for_each(|block| {
            write_little_endian(&mut buf, block.magic_0);
            write_little_endian(&mut buf, block.magic_1);
            write_little_endian(&mut buf, block.flags);
            write_little_endian(&mut buf, block.target_addr);
            write_little_endian(&mut buf, block.payload_size);
            write_little_endian(&mut buf, block.block_no);
            write_little_endian(&mut buf, block.num_blocks);
            write_little_endian(&mut buf, block.file_size);
            let remaining = 476 - block.data.len();
            buf.append(&mut block.data.clone());
            for _ in 0..remaining {
                buf.push(0u8);
            }
            write_little_endian(&mut buf, block.magic_end);
        });

        return buf;
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Read the input file
    let mut inp_file = File::open(args.bootrom)?;
    let mut prog_file = File::open(args.progdata)?;

    // Create a blank payload=
    let mut data_buffer = Vec::new();
    inp_file.read_to_end(&mut data_buffer);

    for _ in data_buffer.len()..256 {
        data_buffer.push(0);
    }

    // Fill the program
    prog_file.read_to_end(&mut data_buffer);

    // Create the uf2
    let uf2_file = Uf2::create(&data_buffer.as_slice());

    // Write the uf2
    let mut file = File::create(args.output)?;
    file.write(uf2_file.as_bytes().as_slice());
    return Ok(());
}
