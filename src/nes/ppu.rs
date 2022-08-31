use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::nes::bus::Bus;

pub struct PPU
{
	screen_x: u16,
	screen_y: u16,
	new_frame: bool,

	bus: Weak<RefCell<Bus>>
}

impl PPU 
{
	pub fn new(bus: &Rc<RefCell<Bus>>) -> PPU 
	{
		PPU {
			screen_x: 0,
			screen_y: 0,
			new_frame: false,

			bus: Rc::downgrade(bus)
		}
	}

	pub fn set_regsiter(&mut self, addr: u16, val: u8)
	{
		match addr 
		{
			_ => panic!("Register not implemented")
		}
	}

	pub fn get_regsiter(&mut self, addr: u16) -> u8
	{
		match addr 
		{
			_ => panic!("Register not implemented")
		}
	}

	pub fn dot(&mut self)
	{
		self.screen_x += 1;

		if self.screen_x > 340 {
			self.screen_x = 0;
			self.screen_y += 1;

			if self.screen_y > 261 {
				self.screen_y = 0;
			}
		}
	}

	pub fn sync(&mut self) -> bool
	{
		if self.new_frame {
			self.new_frame = false;
			return true;
		}

		false
	}

	pub fn current_dot(&self) -> (u16, u16)
	{
		(self.screen_x, self.screen_y)
	}
}