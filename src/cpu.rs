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

pub enum FetchType
{
	Acc,
	Mem
}

pub struct CPU
{
	pub cycle: u8,
	total_cycles: u64,

	pub absolute_addr: u16,
	pub relative_addr: i8,
	pub fetch_type: FetchType,

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

		instr_set[0x08] = instr!(php, imp, 3, 1);
		instr_set[0x09] = instr!(ora, imm, 2, 2);
		instr_set[0x0A] = instr!(asl, acc, 2, 1);

		instr_set[0x10] = instr!(bpl, rel, 2, 2);
		instr_set[0x18] = instr!(clc, imp, 2, 1);

		instr_set[0x20] = instr!(jsr, abs, 6, 3);
		instr_set[0x24] = instr!(bit, zpg, 3, 2);
		instr_set[0x28] = instr!(plp, imp, 4, 1);
		instr_set[0x29] = instr!(and, imm, 2, 2);
		instr_set[0x2A] = instr!(rol, acc, 2, 1);

		instr_set[0x30] = instr!(bmi, rel, 2, 2);
		instr_set[0x38] = instr!(sec, imp, 2, 1);

		instr_set[0x40] = instr!(rti, imp, 6, 1);
		instr_set[0x4A] = instr!(lsr, acc, 2, 1);
		instr_set[0x48] = instr!(pha, imp, 3, 1);
		instr_set[0x49] = instr!(eor, imm, 2, 2);
		instr_set[0x4C] = instr!(jmp, abs, 3, 3);

		instr_set[0x50] = instr!(bvc, rel, 2, 2);

		instr_set[0x60] = instr!(rts, imp, 6, 1);
		instr_set[0x68] = instr!(pla, imp, 4, 1);
		instr_set[0x69] = instr!(adc, imm, 2, 2);
		instr_set[0x6A] = instr!(ror, acc, 2, 1);
		
		instr_set[0x70] = instr!(bvs, rel, 2, 2);
		instr_set[0x78] = instr!(sei, imp, 2, 1);

		instr_set[0x84] = instr!(sty, zpg, 3, 2);
		instr_set[0x85] = instr!(sta, zpg, 3, 2);
		instr_set[0x86] = instr!(stx, zpg, 3, 2);
		instr_set[0x88] = instr!(dey, imp, 2, 1);
		instr_set[0x8A] = instr!(txa, imp, 2, 1);
		instr_set[0x8D] = instr!(sta, abs, 4, 3);
		instr_set[0x8E] = instr!(stx, abs, 4, 3);

		instr_set[0x90] = instr!(bcc, rel, 2, 2);
		instr_set[0x98] = instr!(tya, imp, 2, 1);
		instr_set[0x9A] = instr!(txs, imp, 2, 1);

		instr_set[0xA0] = instr!(ldy, imm, 2, 2);
		instr_set[0xA2] = instr!(ldx, imm, 2, 2);
		instr_set[0xA5] = instr!(lda, zpg, 3, 2);
		instr_set[0xA8] = instr!(tay, imp, 2, 1);
		instr_set[0xA9] = instr!(lda, imm, 2, 2);
		instr_set[0xAA] = instr!(tax, imp, 2, 1);
		instr_set[0xAD] = instr!(lda, abs, 4, 3);
		instr_set[0xAE] = instr!(ldx, abs, 4, 3);
		
		instr_set[0xB0] = instr!(bcs, rel, 2, 2);
		instr_set[0xB8] = instr!(clv, imp, 2, 1);
		instr_set[0xBA] = instr!(tsx, imp, 2, 1);

		instr_set[0xC0] = instr!(cpy, imm, 2, 2);
		instr_set[0xC8] = instr!(iny, imp, 2, 1);
		instr_set[0xC9] = instr!(cmp, imm, 2, 2);
		instr_set[0xCA] = instr!(dex, imp, 2, 1);

		instr_set[0xD0] = instr!(bne, rel, 2, 2);
		instr_set[0xD8] = instr!(cld, imp, 2, 1);

		instr_set[0xE0] = instr!(cpx, imm, 2, 2);
		instr_set[0xE8] = instr!(inx, imp, 2, 1);
		instr_set[0xE9] = instr!(sbc, imm, 2, 2);
		instr_set[0xEA] = instr!(nop, imp, 2, 1);

		instr_set[0xF0] = instr!(beq, rel, 2, 2);
		instr_set[0xF8] = instr!(sed, imp, 2, 1);

		CPU {
			cycle: 0,
			total_cycles: 0,

			absolute_addr: 0,
			relative_addr: 0,
			fetch_type: FetchType::Mem,

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