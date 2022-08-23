use std::{fs::{self, File}, io::{BufReader, Read}};

struct Header 
{
	prg_blocks: u8,
	chr_blocks: u8
}

pub struct Cartridge
{
	header: Header,

	prg: Vec<u8>,
	chr: Vec<u8>
}

impl Cartridge 
{
	pub fn new(filepath: &str) -> Cartridge 
	{
		let fp = File::open(filepath).expect("Failed to load ROM");
		let mut reader = BufReader::new(fp);

		let mut header_data = vec![0u8; 16];
		reader.read_exact(&mut header_data).expect("Header not present in ROM");

		let header = Header {
			prg_blocks: header_data[4],
			chr_blocks: header_data[5]
		};

		// TODO: For now assume there is no trainer
		let mut prg_data = vec![0u8; 0x4000 * header.prg_blocks as usize];
		let mut chr_data = vec![0u8; 0x2000 * header.chr_blocks as usize];

		reader.read_exact(&mut prg_data).expect("ROM does not contain specified amount of PRG data");
		reader.read_exact(&mut chr_data).expect("ROM does not contain specified amount of CHR data");

		Cartridge 
		{ 
			header: header,

			prg: prg_data, 
			chr: chr_data
		}
	}

	// TODO: For now all memio is hardcoded to work with nestest.nes for testing

	pub fn read_prg(&self, addr: u16) -> u8 
	{
		self.prg[(addr & 0x3FFF) as usize]
	}
}