use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::bus::Bus;
use crate::mnemonic::Mnemonic;

type InstrFn = fn(&mut CPU);
type AddrFn = fn(&mut CPU);

#[derive(Clone, Copy)]
struct Instruction
{
	action: InstrFn,
	addressing: AddrFn,
	cycles: u8,
	length: u8,

	name: Mnemonic
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

			name: Mnemonic::new(stringify!($instr), false)
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
}

impl CPU 
{
	pub fn new(bus: &Rc<RefCell<Bus>>) -> CPU 
	{
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

			bus: Rc::downgrade(bus)
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
		let instr = INSTRUCTION_SET[opcode as usize].expect(&format!("Unimplemented opcode {:02X}", opcode));

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

static INSTRUCTION_SET: [Option<Instruction>; 256] = [
		/* 00 */ Option::None,
		/* 01 */ instr!(ora, idx, 6, 2),
		/* 02 */ Option::None,
		/* 03 */ Option::None,
		/* 04 */ Option::None,
		/* 05 */ Option::None,
		/* 06 */ Option::None,
		/* 07 */ Option::None,
		/* 08 */ instr!(php, imp, 3, 1),
		/* 09 */ instr!(ora, imm, 2, 2),
		/* 0A */ instr!(asl, acc, 2, 1),
		/* 0B */ Option::None,
		/* 0C */ Option::None,
		/* 0D */ Option::None,
		/* 0E */ Option::None,
		/* 0F */ Option::None,

		/* 10 */ instr!(bpl, rel, 2, 2),
		/* 11 */ Option::None,
		/* 12 */ Option::None,
		/* 13 */ Option::None,
		/* 14 */ Option::None,
		/* 15 */ Option::None,
		/* 16 */ Option::None,
		/* 17 */ Option::None,
		/* 18 */ instr!(clc, imp, 2, 1),
		/* 09 */ Option::None,
		/* 0A */ Option::None,
		/* 0B */ Option::None,
		/* 0C */ Option::None,
		/* 0D */ Option::None,
		/* 0E */ Option::None,
		/* 0F */ Option::None,

		/* 20 */ instr!(jsr, abs, 6, 3),
		/* 21 */ instr!(and, idx, 6, 2),
		/* 22 */ Option::None,
		/* 23 */ Option::None,
		/* 24 */ instr!(bit, zpg, 3, 2),
		/* 25 */ Option::None,
		/* 26 */ Option::None,
		/* 27 */ Option::None,
		/* 28 */ instr!(plp, imp, 4, 1),
		/* 29 */ instr!(and, imm, 2, 2),
		/* 2A */ instr!(rol, acc, 2, 1),
		/* 2B */ Option::None,
		/* 2C */ Option::None,
		/* 2D */ Option::None,
		/* 2E */ Option::None,
		/* 2F */ Option::None,

		/* 30 */ instr!(bmi, rel, 2, 2),
		/* 31 */ Option::None,
		/* 32 */ Option::None,
		/* 33 */ Option::None,
		/* 34 */ Option::None,
		/* 35 */ Option::None,
		/* 36 */ Option::None,
		/* 37 */ Option::None,
		/* 38 */ instr!(sec, imp, 2, 1),
		/* 39 */ Option::None,
		/* 3A */ Option::None,
		/* 3B */ Option::None,
		/* 3C */ Option::None,
		/* 3D */ Option::None,
		/* 3E */ Option::None,
		/* 3F */ Option::None,

		/* 40 */ instr!(rti, imp, 6, 1),
		/* 41 */ instr!(eor, idx, 6, 2),
		/* 42 */ Option::None,
		/* 43 */ Option::None,
		/* 44 */ Option::None,
		/* 45 */ Option::None,
		/* 46 */ Option::None,
		/* 47 */ Option::None,
		/* 48 */ instr!(pha, imp, 3, 1),
		/* 49 */ instr!(eor, imm, 2, 2),
		/* 4A*/  instr!(lsr, acc, 2, 1),
		/* 4B */ Option::None,
		/* 4C */ instr!(jmp, abs, 3, 3),
		/* 4D */ Option::None,
		/* 4E */ Option::None,
		/* 4F */ Option::None,

		/* 50 */ instr!(bvc, rel, 2, 2),
		/* 51 */ Option::None,
		/* 52 */ Option::None,
		/* 53 */ Option::None,
		/* 54 */ Option::None,
		/* 55 */ Option::None,
		/* 56 */ Option::None,
		/* 57 */ Option::None,
		/* 58 */ Option::None,
		/* 59 */ Option::None,
		/* 5A */ Option::None,
		/* 5B */ Option::None,
		/* 5C */ Option::None,
		/* 5D */ Option::None,
		/* 5E */ Option::None,
		/* 5F */ Option::None,

		/* 60 */ instr!(rts, imp, 6, 1),
		/* 61 */ instr!(adc, idx, 6, 2),
		/* 62 */ Option::None,
		/* 63 */ Option::None,
		/* 64 */ Option::None,
		/* 65 */ Option::None,
		/* 66 */ Option::None,
		/* 67 */ Option::None,
		/* 68 */ instr!(pla, imp, 4, 1),
		/* 69 */ instr!(adc, imm, 2, 2),
		/* 6A */ instr!(ror, acc, 2, 1),
		/* 6B */ Option::None,
		/* 6C */ Option::None,
		/* 6D */ Option::None,
		/* 6E */ Option::None,
		/* 6F */ Option::None,
		
		/* 70 */ instr!(bvs, rel, 2, 2),
		/* 71 */ Option::None,
		/* 72 */ Option::None,
		/* 73 */ Option::None,
		/* 74 */ Option::None,
		/* 75 */ Option::None,
		/* 76 */ Option::None,
		/* 77 */ Option::None,
		/* 78 */ instr!(sei, imp, 2, 1),
		/* 79 */ Option::None,
		/* 7A */ Option::None,
		/* 7B */ Option::None,
		/* 7C */ Option::None,
		/* 7D */ Option::None,
		/* 7E */ Option::None,
		/* 7F */ Option::None,

		/* 80 */ Option::None,
		/* 81 */ instr!(sta, idx, 6, 2),
		/* 82 */ Option::None,
		/* 83 */ Option::None,
		/* 84 */ instr!(sty, zpg, 3, 2),
		/* 85 */ instr!(sta, zpg, 3, 2),
		/* 86 */ instr!(stx, zpg, 3, 2),
		/* 87 */ Option::None,
		/* 88 */ instr!(dey, imp, 2, 1),
		/* 89 */ Option::None,
		/* 8A */ instr!(txa, imp, 2, 1),
		/* 8B */ Option::None,
		/* 8C */ Option::None,
		/* 8D */ instr!(sta, abs, 4, 3),
		/* 8E */ instr!(stx, abs, 4, 3),
		/* 8F */ Option::None,

		/* 90 */ instr!(bcc, rel, 2, 2),
		/* 91 */ Option::None,
		/* 92 */ Option::None,
		/* 93 */ Option::None,
		/* 94 */ Option::None,
		/* 95 */ Option::None,
		/* 96 */ Option::None,
		/* 97 */ Option::None,
		/* 98 */ instr!(tya, imp, 2, 1),
		/* 99 */ Option::None,
		/* 9A */ instr!(txs, imp, 2, 1),
		/* 9B */ Option::None,
		/* 9C */ Option::None,
		/* 9D */ Option::None,
		/* 9E */ Option::None,
		/* 9F */ Option::None,

		/* A0 */ instr!(ldy, imm, 2, 2),
		/* A1 */ instr!(lda, idx, 6, 2),
		/* A2 */ instr!(ldx, imm, 2, 2),
		/* A3 */ Option::None,
		/* A4 */ instr!(ldy, zpg, 3, 2),
		/* A5 */ instr!(lda, zpg, 3, 2),
		/* A6 */ instr!(ldx, zpg, 3, 2),
		/* A7 */ Option::None,
		/* A8 */ instr!(tay, imp, 2, 1),
		/* A9 */ instr!(lda, imm, 2, 2),
		/* AA */ instr!(tax, imp, 2, 1),
		/* AB */ Option::None,
		/* AC */ Option::None,
		/* AD */ instr!(lda, abs, 4, 3),
		/* AE */ instr!(ldx, abs, 4, 3),
		/* AF */ Option::None,
		
		/* B0 */ instr!(bcs, rel, 2, 2),
		/* B1 */ Option::None,
		/* B2 */ Option::None,
		/* B3 */ Option::None,
		/* B4 */ Option::None,
		/* B5 */ Option::None,
		/* B6 */ Option::None,
		/* B7 */ Option::None,
		/* B8 */ instr!(clv, imp, 2, 1),
		/* B9 */ Option::None,
		/* BA */ instr!(tsx, imp, 2, 1),
		/* BB */ Option::None,
		/* BC */ Option::None,
		/* BD */ Option::None,
		/* BE */ Option::None,
		/* BF */ Option::None,

		/* C0 */ instr!(cpy, imm, 2, 2),
		/* C1 */ instr!(cmp, idx, 6, 2),
		/* C2 */ Option::None,
		/* C3 */ Option::None,
		/* C4 */ Option::None,
		/* C5 */ Option::None,
		/* C6 */ Option::None,
		/* C7 */ Option::None,
		/* C8 */ instr!(iny, imp, 2, 1),
		/* C9 */ instr!(cmp, imm, 2, 2),
		/* CA */ instr!(dex, imp, 2, 1),
		/* CB */ Option::None,
		/* CC */ Option::None,
		/* CD */ Option::None,
		/* CE */ Option::None,
		/* CF */ Option::None,

		/* D0 */ instr!(bne, rel, 2, 2),
		/* D1 */ Option::None,
		/* D2 */ Option::None,
		/* D3 */ Option::None,
		/* D4 */ Option::None,
		/* D5 */ Option::None,
		/* D6 */ Option::None,
		/* D7 */ Option::None,
		/* D8 */ instr!(cld, imp, 2, 1),
		/* D9 */ Option::None,
		/* DA */ Option::None,
		/* DB */ Option::None,
		/* DC */ Option::None,
		/* DD */ Option::None,
		/* DE */ Option::None,
		/* DF */ Option::None,

		/* E0 */ instr!(cpx, imm, 2, 2),
		/* E1 */ instr!(sbc, idx, 6, 2),
		/* E2 */ Option::None,
		/* E3 */ Option::None,
		/* E4 */ Option::None,
		/* E5 */ Option::None,
		/* E6 */ Option::None,
		/* E7 */ Option::None,
		/* E8 */ instr!(inx, imp, 2, 1),
		/* E9 */ instr!(sbc, imm, 2, 2),
		/* EA */ instr!(nop, imp, 2, 1),
		/* EB */ Option::None,
		/* EC */ Option::None,
		/* ED */ Option::None,
		/* EE */ Option::None,
		/* EF */ Option::None,

		/* F0 */ instr!(beq, rel, 2, 2),
		/* F1 */ Option::None,
		/* F2 */ Option::None,
		/* F3 */ Option::None,
		/* F4 */ Option::None,
		/* F5 */ Option::None,
		/* F6 */ Option::None,
		/* F7 */ Option::None,
		/* F8 */ instr!(sed, imp, 2, 1),
		/* F9 */ Option::None,
		/* FA */ Option::None,
		/* FB */ Option::None,
		/* FC */ Option::None,
		/* FD */ Option::None,
		/* FE */ Option::None,
		/* FF */ Option::None
];