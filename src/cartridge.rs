use std::fs;

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
		let contents = fs::read(filepath).expect("Failed to load ROM");
		let iter = contents.iter();

		let mut curr_pos: usize = 0;

		let header_data = &contents[curr_pos..curr_pos + 16];
		let header = Header {
			prg_blocks: header_data[4],
			chr_blocks: header_data[5]
		};

		// TODO: For now assume there is no trainer
		curr_pos = 16;
		let prg_data = &contents[curr_pos..(curr_pos + 0x4000 * header.prg_blocks as usize)];

		curr_pos += 0x4000 * header.prg_blocks as usize;
		let chr_data = &contents[curr_pos..(curr_pos + 0x2000 * header.chr_blocks as usize)];

		Cartridge 
		{ 
			header: header,

			prg: prg_data.to_vec(), 
			chr: chr_data.to_vec()
		}
	}

	// TODO: For now all memio is hardcoded to work with nestest.nes for testing

	pub fn read_prg(&self, addr: u16) -> u8 
	{
		self.prg[(addr & 0x3FFF) as usize]
	}
}