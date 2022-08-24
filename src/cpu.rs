use std::cell::RefCell;
use std::rc::{Rc, Weak};
use copystr::s3;

use crate::bus::Bus;

type InstrFn = fn(&mut CPU);
type AddrFn = fn(&mut CPU);

#[derive(Clone, Copy)]
struct Instruction
{
	action: InstrFn,
	addressing: AddrFn,
	cycles: u8,
	length: u8,

	name: s3
}

macro_rules! instr 
{
	($instr: ident, $addr: ident, $cyc: literal, $len: literal) =>
	{
		Option::Some(Instruction 
		{
			action: CPU::$instr,
			addressing: CPU::$addr,
			cycles: $cyc,
			length: $len,

			name: s3::new(stringify!($instr)).unwrap()
		})
	}
}

pub struct CPU
{
	pub cycle: u8,
	total_cycles: u64,
	pub absolute_addr: u16,
	pub relative_addr: i8,

	pub acc: u8,
	pub x: u8,
	pub y: u8,
	pub p: u8,
	pub sp: u8,
	pub pc: u16,

	pub bus: Weak<RefCell<Bus>>,

	instruction_set: [Option<Instruction>; 256]
}

impl CPU 
{
	pub fn new(bus: &Rc<RefCell<Bus>>) -> CPU 
	{
		const UNDEF_INSTR: Option<Instruction> = None;
		let mut instr_set = [UNDEF_INSTR; 256];

		instr_set[0x10] = instr!(bpl, rel, 2, 2);
		instr_set[0x18] = instr!(clc, imp, 2, 1);

		instr_set[0x20] = instr!(jsr, abs, 6, 3);
		instr_set[0x24] = instr!(bit, zpg, 3, 2);

		instr_set[0x30] = instr!(bmi, rel, 2, 2);
		instr_set[0x38] = instr!(sec, imp, 2, 1);

		instr_set[0x4C] = instr!(jmp, abs, 3, 3);

		instr_set[0x50] = instr!(bvc, rel, 2, 2);

		instr_set[0x60] = instr!(rts, imp, 6, 1);

		instr_set[0x70] = instr!(bvs, rel, 2, 2);

		instr_set[0x85] = instr!(sta, zpg, 3, 2);
		instr_set[0x86] = instr!(stx, zpg, 3, 2);

		instr_set[0x90] = instr!(bcc, rel, 2, 2);

		instr_set[0xA2] = instr!(ldx, imm, 2, 2);
		instr_set[0xA9] = instr!(lda, imm, 2, 2);
		
		instr_set[0xB0] = instr!(bcs, rel, 2, 2);

		instr_set[0xD0] = instr!(bne, rel, 2, 2);

		instr_set[0xEA] = instr!(nop, imp, 2, 1);

		instr_set[0xF0] = instr!(beq, rel, 2, 2);

		CPU {
			cycle: 0,
			total_cycles: 0,
			absolute_addr: 0,
			relative_addr: 0,

			acc: 0,
			x: 0,
			y: 0,
			p: 0,
			sp: 0,

			pc: 0,

			bus: Rc::downgrade(bus),

			instruction_set: instr_set
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
		let instr = self.instruction_set[opcode as usize].expect(&format!("Unimplemented opcode {:02X}", opcode));

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