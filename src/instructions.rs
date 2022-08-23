use crate::cpu::CPU;

#[allow(dead_code)]
enum Bit
{
	Negative 	= 7,
	Overflow 	= 6,
	Decimal 	= 3,
	Interrupt 	= 2,
	Zero		= 1,
	Carry	 	= 0,
}

macro_rules! set_flag
{
	($target: expr, $flag: expr) => 
	{
		$target |= (1u8 << ($flag as u8))
	}
}

macro_rules! clear_flag
{
	($target: expr, $flag: expr) => 
	{
		$target &= !(1u8 << ($flag as u8))
	}
}

macro_rules! set_flag_to
{
	($target: expr, $flag: expr, $value: expr) => 
	{
		$target = ($target & !(1u8 << ($flag as u8))) | (($value as u8) << ($flag as u8));
	}
}

impl CPU 
{
	fn fetch(&mut self) -> u8
	{
		let bus = self.bus.upgrade().unwrap();
		return bus.borrow().read_cpu(self.absolute_addr);
	}

	pub fn jmp(&mut self)
	{
		self.pc = self.absolute_addr;
	}

	pub fn ldx(&mut self)
	{
		self.x = self.fetch();

		set_flag_to!(self.p, Bit::Negative, (self.x & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.x == 0);
	}

	pub fn stx(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		bus.borrow_mut().write_cpu(self.absolute_addr, self.x);
	}
}