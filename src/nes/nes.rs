use std::cell::RefCell;
use std::rc::Rc;
use crate::nes::bus::Bus;
use crate::nes::cpu::CPU;
use crate::nes::ppu::PPU;

pub struct NES
{
	bus: Rc<RefCell<Bus>>,
	cpu: Rc<RefCell<CPU>>,
	ppu: Rc<RefCell<PPU>>
}

macro_rules! clock 
{
	($cpu: ident, $ppu: ident) =>
	{
		let res = $cpu.cycle();

		$ppu.dot();
		$ppu.dot();
		$ppu.dot();

		if res {
			let (x, y) = $ppu.current_dot();
			println!("PPU:{: <3},{: <3}", y, x);
		}
	}
}

impl NES
{
	pub fn new() -> NES 
	{
		let bus: Rc<RefCell<Bus>> = Rc::new(RefCell::new(Bus::new()));
		let cpu: Rc<RefCell<CPU>> = Rc::new(RefCell::new(CPU::new(&bus)));
		let ppu: Rc<RefCell<PPU>> = Rc::new(RefCell::new(PPU::new(&bus)));

		bus.borrow_mut().attach_cpu(&cpu);
		bus.borrow_mut().attach_ppu(&ppu);

		NES 
		{
			bus: bus,
			cpu: cpu,
			ppu: ppu
		}
	}

	pub fn powerup(&self)
	{
		self.cpu.borrow_mut().powerup();
	}

	pub fn clock(&self)
	{
		let mut cpu = self.cpu.borrow_mut();
		let mut ppu = self.ppu.borrow_mut();

		clock!(cpu, ppu);
	}

	pub fn single_step(&self)
	{
		let mut cpu = self.cpu.borrow_mut();
		let mut ppu = self.ppu.borrow_mut();

		while !cpu.sync() {
			clock!(cpu, ppu);
		}

		clock!(cpu, ppu);
	}

	pub fn single_frame(&self)
	{
		let mut cpu = self.cpu.borrow_mut();
		let mut ppu = self.ppu.borrow_mut();

		while !ppu.sync() {
			clock!(cpu, ppu);
		}
	}
}