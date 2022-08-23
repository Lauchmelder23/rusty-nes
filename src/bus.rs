use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::cpu::CPU;
use crate::cartridge::Cartridge;

pub struct Bus
{
	cpu: Weak<RefCell<CPU>>,
	cartridge: Cartridge,

	ram: Vec<u8>
}

impl Bus 
{
	pub fn new() -> Bus 
	{
		Bus 
		{
			cpu: Weak::new(),
			cartridge: Cartridge::new("roms/nestest.nes"),
			ram: vec![0; 0x800]
		}
	}

	pub fn run(&self)
	{
		let cpu = self.cpu.upgrade().unwrap();

		loop
		{
			cpu.borrow_mut().cycle();
		}
	}

	pub fn attach_cpu(&mut self, cpu: &Rc<RefCell<CPU>>)
	{	
		self.cpu = Rc::downgrade(cpu);
	}

	pub fn read_cpu(&self, addr: u16) -> u8 
	{
		match addr
		{
			0..=0x1FFF 		=> self.ram[(addr & 0x7FF) as usize],
			0x8000..=0xFFFF => self.cartridge.read_prg(addr & 0x7FFF),

			_ => panic!("Tried to access invalid memory address {}", addr)
		}
	}
}