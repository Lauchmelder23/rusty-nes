use crate::cpu::{CPU, FetchType};

#[macro_export]
macro_rules! instr_size
{
	(acc) => { 1 };
	(abs) => { 3 };
	(abx) => { 3 };
	(aby) => { 3 };
	(imm) => { 1 };
	(imp) => { 1 };
	(ind) => { 3 };
	(idx) => { 3 };
	(idy) => { 3 };
	(rel) => { 2 };
	(zpg) => { 2 };
	(zpx) => { 2 };
	(zpy) => { 2 };
}

pub type AddrFn = fn(&mut CPU);

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

		print!("{: <40}", format!("${:04X}", self.absolute_addr));
	}

	pub fn acc(&mut self)
	{
		self.fetch_type = FetchType::Acc;

		print!("{: <40}", "A");
	}

	pub fn idx(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		
		let mut zpg_addr = bus.borrow().read_cpu(self.pc);
		self.pc += 1;

		zpg_addr = zpg_addr.wrapping_add(self.x);
		let lo = bus.borrow().read_cpu(zpg_addr as u16) as u16;
		let hi = bus.borrow().read_cpu(zpg_addr.wrapping_add(1) as u16) as u16;

		self.absolute_addr = (hi << 8) | lo;
		self.fetch_type = FetchType::Mem;

		print!("{: <40}", format!("(${:02X},X) @ [${:02X}] = ${:04X} = {:02X}", zpg_addr.wrapping_sub(self.x), zpg_addr, self.absolute_addr, bus.borrow().read_cpu(self.absolute_addr)));
	}

	pub fn idy(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		
		let zpg_addr = bus.borrow().read_cpu(self.pc);
		self.pc += 1;

		let lo = bus.borrow().read_cpu(zpg_addr as u16) as u16;
		let hi = bus.borrow().read_cpu(zpg_addr.wrapping_add(1) as u16) as u16;

		let target_addr = (hi << 8) | lo;
		self.absolute_addr = target_addr.wrapping_add(self.y as u16);
		self.fetch_type = FetchType::Mem;

		print!("{: <40}", format!("(${:02X}),Y @ [${:04X} + Y] = ${:04X} = {:02X}", zpg_addr, target_addr, self.absolute_addr, bus.borrow().read_cpu(self.absolute_addr)));
	}

	pub fn imm(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.absolute_addr = self.pc;
		self.pc += 1;

		self.fetch_type = FetchType::Mem;

		print!("{: <40}", format!("#${:02X}", bus.borrow().read_cpu(self.absolute_addr)));
	}

	pub fn imp(&mut self)
	{
		print!("{: <40}", "");
	}

	pub fn rel(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		
		self.relative_addr = bus.borrow().read_cpu(self.pc) as i8;
		self.pc += 1;

		print!("{: <40}", format!("${:02X}", self.relative_addr));
	}

	pub fn zpg(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.absolute_addr = bus.borrow().read_cpu(self.pc) as u16;
		self.pc += 1;

		self.fetch_type = FetchType::Mem;

		print!("{: <40}", format!("${:02X} = {:02X}", self.absolute_addr, bus.borrow().read_cpu(self.absolute_addr)))
	}
}