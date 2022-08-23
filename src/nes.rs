use std::cell::RefCell;
use std::rc::Rc;
use crate::bus::Bus;
use crate::cpu::CPU;

pub struct NES
{
	bus: Rc<RefCell<Bus>>,
	cpu: Rc<RefCell<CPU>>
}

impl NES
{
	pub fn new() -> NES 
	{
		let bus: Rc<RefCell<Bus>> = Rc::new(RefCell::new(Bus::new()));
		let cpu: Rc<RefCell<CPU>> = Rc::new(RefCell::new(CPU::new(&bus)));

		bus.borrow_mut().attach_cpu(&cpu);

		NES 
		{
			bus: bus,
			cpu: cpu
		}
	}

	pub fn powerup(&self)
	{
		self.cpu.borrow_mut().powerup();

		self.bus.borrow().run();
	}
}