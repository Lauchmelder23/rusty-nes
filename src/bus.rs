use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::cpu::CPU;

pub struct Bus
{
	cpu: Weak<RefCell<CPU>>,

	ram: Vec<u8>
}

impl Bus 
{
	pub fn new() -> Bus 
	{
		Bus 
		{
			cpu: Weak::new(),
			ram: vec![0; 0x800]
		}
	}

	pub fn run(&self)
	{
		let cpu = self.cpu.upgrade().unwrap();

		loop
		{
			cpu.borrow_mut().execute();
		}
	}

	pub fn attach_cpu(&mut self, cpu: &Rc<RefCell<CPU>>)
	{	
		self.cpu = Rc::downgrade(cpu);
	}

	pub fn read_cpu(&self, addr: u16) -> u8 
	{
		self.ram[addr as usize]
	}
}