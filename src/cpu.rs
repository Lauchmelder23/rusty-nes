use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::bus::Bus;
use crate::instructions::INSTRUCTION_SET;

pub enum FetchType
{
	Acc,
	Mem
}

pub struct CPU
{
	pub cycle: u8,
	total_cycles: u64,

	pub absolute_addr: u16,
	pub relative_addr: i8,
	pub fetch_type: FetchType,

	pub acc: u8,
	pub x: u8,
	pub y: u8,
	pub p: u8,
	pub sp: u8,
	pub pc: u16,

	pub bus: Weak<RefCell<Bus>>,
}

impl CPU 
{
	pub fn new(bus: &Rc<RefCell<Bus>>) -> CPU 
	{
		CPU {
			cycle: 0,
			total_cycles: 0,

			absolute_addr: 0,
			relative_addr: 0,
			fetch_type: FetchType::Mem,

			acc: 0,
			x: 0,
			y: 0,
			p: 0,
			sp: 0,

			pc: 0,

			bus: Rc::downgrade(bus)
		}
	}

	pub fn powerup(&mut self)
	{
		self.p = 0x34;

		self.acc = 0;
		self.x = 0;
		self.y = 0;
		self.sp = 0xFD;

		self.total_cycles = 0;
		self.cycle = 6;

		// TODO: This is just for the nestest.nes
		self.pc = 0xC000;
	}

	pub fn cycle(&mut self)
	{
		self.total_cycles += 1;

		if self.cycle > 0
		{
			self.cycle -= 1;
			return;
		}

		self.execute();

		self.cycle -= 1;
	}

	fn execute(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		let opcode: u8 = bus.borrow().read_cpu(self.pc);
		let instr = INSTRUCTION_SET[opcode as usize].expect(&format!("Unimplemented opcode {:02X}", opcode));

		print!("{:04X}  ", self.pc);
		for byte in 0..3
		{
			if byte < instr.length 
			{
				print!("{:02X} ", bus.borrow().read_cpu(self.pc + byte as u16));
			}
			else
			{
				print!("   ");	
			}
		}

		print!(" {} ", instr.name.to_string().to_uppercase());

		self.pc += 1;

		(instr.addressing)(self);
		(instr.action)(self);

		self.cycle += instr.cycles;

		println!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}", self.acc, self.x, self.y, self.p, self.sp, self.total_cycles);
	}

}
