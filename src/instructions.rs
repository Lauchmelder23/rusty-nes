use crate::cpu::CPU;

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
	}
}