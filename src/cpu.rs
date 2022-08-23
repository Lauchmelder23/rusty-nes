use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::bus::Bus;

pub struct CPU
{
	acc: u8,
	x: u8,
	y: u8,
	p: u8,
	sp: u8,

	pc: u16,

	bus: Weak<RefCell<Bus>>
}

impl CPU 
{
	pub fn new(bus: &Rc<RefCell<Bus>>) -> CPU 
	{
		CPU {
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

		// TODO: This is just for the nestest.nes
		self.pc = 0xC000;
	}

	pub fn execute(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		let opcode: u8 = bus.borrow().read_cpu(self.pc);
		self.pc += 1;

		match (opcode)
		{
			_ => panic!("Unimplemented opcode {:X}", opcode)
		}
	}
}