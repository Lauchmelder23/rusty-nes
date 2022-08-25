use crate::cpu::{CPU, FetchType};

impl CPU
{
	pub fn abs(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let lo = bus.borrow().read_cpu(self.pc) as u16;
		let hi = bus.borrow().read_cpu(self.pc + 1) as u16;

		self.pc += 2;
		self.absolute_addr = (hi << 8) | lo;

		self.fetch_type = FetchType::Mem;

		print!("{: <30}", format!("${:04X}", self.absolute_addr));
	}

	pub fn acc(&mut self)
	{
		self.fetch_type = FetchType::Acc;

		print!("{: <30}", "A");
	}

	pub fn imm(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.absolute_addr = self.pc;
		self.pc += 1;

		self.fetch_type = FetchType::Mem;

		print!("{: <30}", format!("#${:02X}", bus.borrow().read_cpu(self.absolute_addr)));
	}

	pub fn imp(&mut self)
	{
		print!("{: <30}", "");
	}

	pub fn rel(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		
		self.relative_addr = bus.borrow().read_cpu(self.pc) as i8;
		self.pc += 1;

		print!("{: <30}", format!("${:02X}", self.relative_addr));
	}

	pub fn zpg(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.absolute_addr = bus.borrow().read_cpu(self.pc) as u16;
		self.pc += 1;

		self.fetch_type = FetchType::Mem;

		print!("{: <30}", format!("${:02X} = {:02X}", self.absolute_addr, bus.borrow().read_cpu(self.absolute_addr)))
	}
}