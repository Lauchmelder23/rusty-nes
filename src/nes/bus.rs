use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::nes::cpu::CPU;
use crate::nes::ppu::PPU;
use crate::nes::cartridge::Cartridge;

pub struct Bus
{
	cpu: Weak<RefCell<CPU>>,
	ppu: Weak<RefCell<PPU>>,
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
			ppu: Weak::new(),
			cartridge: Cartridge::new("roms/nestest.nes"),
			ram: vec![0; 0x800]
		}
	}

	pub fn attach_cpu(&mut self, cpu: &Rc<RefCell<CPU>>)
	{	
		self.cpu = Rc::downgrade(cpu);
	}

	pub fn attach_ppu(&mut self, ppu: &Rc<RefCell<PPU>>)
	{	
		self.ppu = Rc::downgrade(ppu);
	}

	pub fn read_cpu(&self, addr: u16) -> u8 
	{
		match addr
		{
			0..=0x1FFF 		=> self.ram[(addr & 0x7FF) as usize],
			0x2000..=0x3FFF => self.ppu.upgrade().unwrap().borrow_mut().get_regsiter(addr & 0x7),
			0x8000..=0xFFFF => self.cartridge.read_prg(addr & 0x7FFF),

			_ => panic!("Tried to access invalid memory address ${:04X}", addr)
		}
	}

	pub fn write_cpu(&mut self, addr: u16, val: u8) 
	{
		match addr 
		{
			0..=0x1FFF 		=> self.ram[(addr & 0x7FF) as usize] = val,
			0x2000..=0x3FFF => self.ppu.upgrade().unwrap().borrow_mut().set_regsiter(addr & 0x7, val),
			0x8000..=0xFFFF => self.cartridge.write_prg(addr & 0x7FFF, val),

			_ => { }
		}
	}
}