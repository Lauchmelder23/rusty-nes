use crate::cpu::{CPU, FetchType};
use crate::bus::Bus;
use crate::addressing::AddrFn;
use crate::mnemonic::Mnemonic;
use crate::instr_size;
use std::cell::Ref;

pub type InstrFn = fn(&mut CPU);

#[derive(Clone, Copy)]
pub struct Instruction
{
	pub action: InstrFn,
	pub addressing: AddrFn,
	pub cycles: u8,
	pub length: u8,

	pub name: Mnemonic
}

macro_rules! instr 
{
	($instr: ident, $addr: ident, $cyc: literal) =>
	{
		Option::Some(Instruction 
		{
			action: CPU::$instr,
			addressing: CPU::$addr,
			cycles: $cyc,
			length: instr_size!($addr),

			name: Mnemonic::new(stringify!($instr), false)
		})
	}
}

#[allow(dead_code)]
enum Bit
{
	Negative 	= 7,
	Overflow 	= 6,
	Break		= 4,
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

macro_rules! test_flag
{
	($target: expr, $flag: expr) => 
	{
		($target & (1u8 << ($flag as u8))) != 0
	}
}

macro_rules! push
{
	($bus: expr, $sp: expr, $val: expr) =>
	{
		$bus.write_cpu(0x0100 + $sp as u16, ($val & 0xFF) as u8);
		$sp -= 1;
	}
}

fn pop(bus: Ref<Bus>, sp: &mut u8) -> u8 
{
	*sp += 1;
	bus.read_cpu(0x0100 + *sp as u16)
}

macro_rules! branch
{
	($self: ident) => 
	{
		let branch_target = $self.pc.wrapping_add($self.relative_addr as u16);
			
		$self.cycle += 1;
		if (branch_target & 0xFF00) != ($self.pc & 0xFF00)	// Branched to different page
		{
			$self.cycle += 1;
		}

		$self.pc = branch_target;
	}
}

macro_rules! branch_on_fn
{
	($name: ident, $flag: expr, $result: literal) => 
	{
		pub fn $name(&mut self)
		{
			if test_flag!(self.p, $flag) == $result
			{
				branch!(self);
			}
		}
	}
}

macro_rules! set_flag_fn
{
	($name: ident, $flag: expr, $result: literal) => 
	{
		pub fn $name(&mut self)
		{
			match $result 
			{
				false 	=> clear_flag!(self.p, $flag),
				true 	=> set_flag!(self.p, $flag)
			}
		}
	}
}

macro_rules! load_fn
{
	($name: ident, $register: ident) => 
	{
		pub fn $name(&mut self) 
		{
			self.$register = self.fetch();

			set_flag_to!(self.p, Bit::Negative, (self.$register & (1u8 << 7)) > 0);
			set_flag_to!(self.p, Bit::Zero, self.$register == 0);
		}
	};
}

macro_rules! store_fn
{
	($name: ident, $register: ident) => 
	{
		pub fn $name(&mut self) 
		{
			let bus = self.bus.upgrade().unwrap();
			bus.borrow_mut().write_cpu(self.absolute_addr, self.$register);
		}
	};
}

macro_rules! transfer_fn
{
	($name: ident, $from: ident, $to: ident) => 
	{
		pub fn $name(&mut self)
		{
			self.$to = self.$from;

			match stringify!($to)
			{
				"sp" => {},
				_ => {
					set_flag_to!(self.p, Bit::Negative, (self.$to >> 7) == 0x01);
					set_flag_to!(self.p, Bit::Zero, self.$to == 0);
				}
			};
		}
	}
}

macro_rules! inc_dec_fn
{
	($name: ident, $register: ident, $increment: literal) =>
	{
		pub fn $name(&mut self)
		{
			match $increment 
			{
				false 	=> self.$register = self.$register.wrapping_sub(1),
				true 	=> self.$register = self.$register.wrapping_add(1)
			}

			set_flag_to!(self.p, Bit::Negative, (self.$register >> 7) == 1);
			set_flag_to!(self.p, Bit::Zero, self.$register == 0);
		}
	};

	($name: ident, $increment: literal) =>
	{
		pub fn $name(&mut self)
		{
			let bus = self.bus.upgrade().unwrap();
			let mut value = self.fetch();

			match $increment 
			{
				false 	=> value = value.wrapping_sub(1),
				true 	=> value = value.wrapping_add(1)
			}

			set_flag_to!(self.p, Bit::Negative, (value >> 7) == 1);
			set_flag_to!(self.p, Bit::Zero, value == 0);

			bus.borrow_mut().write_cpu(self.absolute_addr, value);
		}
	};
}

macro_rules! cmp_fn
{
	($name: ident, $register: ident) => 
	{
		pub fn $name(&mut self)
		{
			let value = self.fetch();
			let result = self.$register.wrapping_sub(value);

			set_flag_to!(self.p, Bit::Zero, self.$register == value);
			set_flag_to!(self.p, Bit::Carry, self.$register >= value);
			set_flag_to!(self.p, Bit::Negative, (result >> 7) != 0);
		}
	}
}

macro_rules! carry_condition
{
	($val: ident, <<) => { ($val & 0x80) == 0x80 };
	($val: ident, >>) => { ($val & 0x01) == 0x01 };
}

macro_rules! handle_carry
{
	($val: ident, $carry: ident, <<) => { $val |= ($carry as u8); };
	($val: ident, $carry: ident, >>) => { $val |= (($carry as u8) << 7); };
}

macro_rules! bitshift_fn 
{
	($name: ident, $direction: tt, $rotate: literal) =>
	{
		pub fn $name(&mut self)
		{
			let mut val = self.fetch();

			let carry = test_flag!(self.p, Bit::Carry);
			set_flag_to!(self.p, Bit::Carry, carry_condition!(val, $direction));

			val = val $direction 1;
			match $rotate
			{
				false => { },
				true  => { handle_carry!(val, carry, $direction); }
			};
	
			set_flag_to!(self.p, Bit::Zero, val == 0x00);
			set_flag_to!(self.p, Bit::Negative, (val & 0x80) == 0x80);
	
			self.ditch(val);
		}
	}
}

impl CPU 
{
	fn fetch(&mut self) -> u8
	{
		match self.fetch_type
		{
			FetchType::Mem => {
				let bus = self.bus.upgrade().unwrap();
				return bus.borrow().read_cpu(self.absolute_addr);
			},

			FetchType::Acc => {
				self.acc
			}
		}
	}

	fn ditch(&mut self, value: u8)
	{
		match self.fetch_type
		{
			FetchType::Mem => {
				let bus = self.bus.upgrade().unwrap();
				bus.borrow_mut().write_cpu(self.absolute_addr, value);
			},

			FetchType::Acc => {
				self.acc = value;
			}
		}
	}

	branch_on_fn!(bcc, Bit::Carry, 		false);
	branch_on_fn!(bcs, Bit::Carry, 		true);
	branch_on_fn!(bne, Bit::Zero, 		false);
	branch_on_fn!(beq, Bit::Zero, 		true);
	branch_on_fn!(bpl, Bit::Negative, 	false);
	branch_on_fn!(bmi, Bit::Negative, 	true);
	branch_on_fn!(bvc, Bit::Overflow, 	false);
	branch_on_fn!(bvs, Bit::Overflow, 	true);

	set_flag_fn!(clc, Bit::Carry, 		false);
	set_flag_fn!(sec, Bit::Carry, 		true);
	set_flag_fn!(sei, Bit::Interrupt, 	true);
	set_flag_fn!(cli, Bit::Interrupt, 	false);
	set_flag_fn!(sed, Bit::Decimal, 	true);
	set_flag_fn!(cld, Bit::Decimal, 	false);
	set_flag_fn!(clv, Bit::Overflow, 	false);

	load_fn!(lda, acc);
	load_fn!(ldx, x);
	load_fn!(ldy, y);

	store_fn!(sta, acc);
	store_fn!(stx, x);
	store_fn!(sty, y);

	transfer_fn!(tax, acc, x);
	transfer_fn!(tay, acc, y);
	transfer_fn!(tsx, sp, x);
	transfer_fn!(txa, x, acc);
	transfer_fn!(tya, y, acc);
	transfer_fn!(txs, x, sp);

	cmp_fn!(cmp, acc);
	cmp_fn!(cpx, x);
	cmp_fn!(cpy, y);

	inc_dec_fn!(inc, 	true);
	inc_dec_fn!(inx, x, true);
	inc_dec_fn!(iny, y, true);

	inc_dec_fn!(dec, 	false);
	inc_dec_fn!(dex, x, false);
	inc_dec_fn!(dey, y, false);

	bitshift_fn!(asl, <<, false);
	bitshift_fn!(lsr, >>, false);
	bitshift_fn!(rol, <<, true);
	bitshift_fn!(ror, >>, true);

	fn adc(&mut self)
	{
		let value = self.fetch() as u16;
		let result = (self.acc as u16) + value + (test_flag!(self.p, Bit::Carry) as u16);

		set_flag_to!(self.p, Bit::Carry, (result & 0xFF00) != 0x0000);
		set_flag_to!(self.p, Bit::Negative, ((result >> 7) & 0x0001) == 0x0001);
		set_flag_to!(self.p, Bit::Zero, (result & 0x00FF) == 0x0000);
		set_flag_to!(self.p, Bit::Overflow, ((result ^ value) & (result ^ self.acc as u16) & 0x80) == 0x80);

		self.acc = result as u8;
	}

	fn sbc(&mut self)
	{
		let value = !(self.fetch() as u16);
		let result = (self.acc as u16).wrapping_add(value).wrapping_add(test_flag!(self.p, Bit::Carry) as u16);

		set_flag_to!(self.p, Bit::Carry, (result & 0xFF00) == 0x0000);
		set_flag_to!(self.p, Bit::Negative, ((result >> 7) & 0x0001) == 0x0001);
		set_flag_to!(self.p, Bit::Zero, (result & 0x00FF) == 0x0000);
		set_flag_to!(self.p, Bit::Overflow, ((result ^ value) & (result ^ self.acc as u16) & 0x80) == 0x80);

		self.acc = result as u8;
	}

	fn and(&mut self)
	{
		let val = self.fetch();

		self.acc &= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	fn bit(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		let value = bus.borrow().read_cpu(self.absolute_addr);

		set_flag_to!(self.p, Bit::Negative, (value >> 7) & 0x1);
		set_flag_to!(self.p, Bit::Overflow, (value >> 6) & 0x1);
		set_flag_to!(self.p, Bit::Zero, (self.acc & value) == 0);
	}

	fn eor(&mut self)
	{
		let val = self.fetch();

		self.acc ^= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	fn jmp(&mut self)
	{
		self.pc = self.absolute_addr;
	}

	fn jsr(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.pc -= 1;
		push!(bus.borrow_mut(), self.sp, self.pc >> 8);
		push!(bus.borrow_mut(), self.sp, self.pc);

		self.pc = self.absolute_addr;
	}

	fn nop(&mut self)
	{
		
	}

	fn ora(&mut self)
	{
		let val = self.fetch();

		self.acc |= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	fn pha(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		push!(bus.borrow_mut(), self.sp, self.acc);
	}

	fn pla(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.acc = pop(bus.borrow(), &mut self.sp);
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	fn php(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let mut value = self.p;
		set_flag!(value, Bit::Break);
		set_flag!(value, 5);

		push!(bus.borrow_mut(), self.sp, value);
	}

	fn plp(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let flag: u8 = pop(bus.borrow(), &mut self.sp);
		let mask: u8 = 0b11001111;

		self.p &= !mask;
		self.p |= flag & mask;
	}

	fn rti(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let flag: u8 = pop(bus.borrow(), &mut self.sp);
		let mask: u8 = 0b11001111;

		self.p &= !mask;
		self.p |= flag & mask;

		let lo = pop(bus.borrow(), &mut self.sp) as u16;
		let hi = pop(bus.borrow(), &mut self.sp) as u16;

		self.pc = (hi << 8) | lo;
	}

	fn rts(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let lo = pop(bus.borrow(), &mut self.sp) as u16;
		let hi = pop(bus.borrow(), &mut self.sp) as u16;

		self.pc = (hi << 8) | lo;
		self.pc += 1;
	}
}


pub static INSTRUCTION_SET: [Option<Instruction>; 256] = [
		/* 00 */ Option::None,
		/* 01 */ instr!(ora, idx, 6),
		/* 02 */ Option::None,
		/* 03 */ Option::None,
		/* 04 */ Option::None,
		/* 05 */ instr!(ora, zpg, 3),
		/* 06 */ instr!(asl, zpg, 5),
		/* 07 */ Option::None,
		/* 08 */ instr!(php, imp, 3),
		/* 09 */ instr!(ora, imm, 2),
		/* 0A */ instr!(asl, acc, 2),
		/* 0B */ Option::None,
		/* 0C */ Option::None,
		/* 0D */ instr!(ora, abs, 4),
		/* 0E */ instr!(asl, abs, 6),
		/* 0F */ Option::None,

		/* 10 */ instr!(bpl, rel, 2),
		/* 11 */ Option::None,
		/* 12 */ Option::None,
		/* 13 */ Option::None,
		/* 14 */ Option::None,
		/* 15 */ Option::None,
		/* 16 */ Option::None,
		/* 17 */ Option::None,
		/* 18 */ instr!(clc, imp, 2),
		/* 09 */ Option::None,
		/* 0A */ Option::None,
		/* 0B */ Option::None,
		/* 0C */ Option::None,
		/* 0D */ Option::None,
		/* 0E */ Option::None,
		/* 0F */ Option::None,

		/* 20 */ instr!(jsr, abs, 6),
		/* 21 */ instr!(and, idx, 6),
		/* 22 */ Option::None,
		/* 23 */ Option::None,
		/* 24 */ instr!(bit, zpg, 3),
		/* 25 */ instr!(and, zpg, 3),
		/* 26 */ instr!(rol, zpg, 5),
		/* 27 */ Option::None,
		/* 28 */ instr!(plp, imp, 4),
		/* 29 */ instr!(and, imm, 2),
		/* 2A */ instr!(rol, acc, 2),
		/* 2B */ Option::None,
		/* 2C */ instr!(bit, abs, 4),
		/* 2D */ instr!(and, abs, 4),
		/* 2E */ instr!(rol, abs, 6),
		/* 2F */ Option::None,

		/* 30 */ instr!(bmi, rel, 2),
		/* 31 */ Option::None,
		/* 32 */ Option::None,
		/* 33 */ Option::None,
		/* 34 */ Option::None,
		/* 35 */ Option::None,
		/* 36 */ Option::None,
		/* 37 */ Option::None,
		/* 38 */ instr!(sec, imp, 2),
		/* 39 */ Option::None,
		/* 3A */ Option::None,
		/* 3B */ Option::None,
		/* 3C */ Option::None,
		/* 3D */ Option::None,
		/* 3E */ Option::None,
		/* 3F */ Option::None,

		/* 40 */ instr!(rti, imp, 6),
		/* 41 */ instr!(eor, idx, 6),
		/* 42 */ Option::None,
		/* 43 */ Option::None,
		/* 44 */ Option::None,
		/* 45 */ instr!(eor, zpg, 3),
		/* 46 */ instr!(lsr, zpg, 5),
		/* 47 */ Option::None,
		/* 48 */ instr!(pha, imp, 3),
		/* 49 */ instr!(eor, imm, 2),
		/* 4A*/  instr!(lsr, acc, 2),
		/* 4B */ Option::None,
		/* 4C */ instr!(jmp, abs, 3),
		/* 4D */ instr!(eor, abs, 4),
		/* 4E */ instr!(lsr, abs, 6),
		/* 4F */ Option::None,

		/* 50 */ instr!(bvc, rel, 2),
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

		/* 60 */ instr!(rts, imp, 6),
		/* 61 */ instr!(adc, idx, 6),
		/* 62 */ Option::None,
		/* 63 */ Option::None,
		/* 64 */ Option::None,
		/* 65 */ instr!(adc, zpg, 3),
		/* 66 */ instr!(ror, zpg, 5),
		/* 67 */ Option::None,
		/* 68 */ instr!(pla, imp, 4),
		/* 69 */ instr!(adc, imm, 2),
		/* 6A */ instr!(ror, acc, 2),
		/* 6B */ Option::None,
		/* 6C */ Option::None,
		/* 6D */ instr!(adc, abs, 4),
		/* 6E */ instr!(ror, abs, 6),
		/* 6F */ Option::None,
		
		/* 70 */ instr!(bvs, rel, 2),
		/* 71 */ Option::None,
		/* 72 */ Option::None,
		/* 73 */ Option::None,
		/* 74 */ Option::None,
		/* 75 */ Option::None,
		/* 76 */ Option::None,
		/* 77 */ Option::None,
		/* 78 */ instr!(sei, imp, 2),
		/* 79 */ Option::None,
		/* 7A */ Option::None,
		/* 7B */ Option::None,
		/* 7C */ Option::None,
		/* 7D */ Option::None,
		/* 7E */ Option::None,
		/* 7F */ Option::None,

		/* 80 */ Option::None,
		/* 81 */ instr!(sta, idx, 6),
		/* 82 */ Option::None,
		/* 83 */ Option::None,
		/* 84 */ instr!(sty, zpg, 3),
		/* 85 */ instr!(sta, zpg, 3),
		/* 86 */ instr!(stx, zpg, 3),
		/* 87 */ Option::None,
		/* 88 */ instr!(dey, imp, 2),
		/* 89 */ Option::None,
		/* 8A */ instr!(txa, imp, 2),
		/* 8B */ Option::None,
		/* 8C */ instr!(sty, abs, 4),
		/* 8D */ instr!(sta, abs, 4),
		/* 8E */ instr!(stx, abs, 4),
		/* 8F */ Option::None,

		/* 90 */ instr!(bcc, rel, 2),
		/* 91 */ Option::None,
		/* 92 */ Option::None,
		/* 93 */ Option::None,
		/* 94 */ Option::None,
		/* 95 */ Option::None,
		/* 96 */ Option::None,
		/* 97 */ Option::None,
		/* 98 */ instr!(tya, imp, 2),
		/* 99 */ Option::None,
		/* 9A */ instr!(txs, imp, 2),
		/* 9B */ Option::None,
		/* 9C */ Option::None,
		/* 9D */ Option::None,
		/* 9E */ Option::None,
		/* 9F */ Option::None,

		/* A0 */ instr!(ldy, imm, 2),
		/* A1 */ instr!(lda, idx, 6),
		/* A2 */ instr!(ldx, imm, 2),
		/* A3 */ Option::None,
		/* A4 */ instr!(ldy, zpg, 3),
		/* A5 */ instr!(lda, zpg, 3),
		/* A6 */ instr!(ldx, zpg, 3),
		/* A7 */ Option::None,
		/* A8 */ instr!(tay, imp, 2),
		/* A9 */ instr!(lda, imm, 2),
		/* AA */ instr!(tax, imp, 2),
		/* AB */ Option::None,
		/* AC */ instr!(ldy, abs, 4),
		/* AD */ instr!(lda, abs, 4),
		/* AE */ instr!(ldx, abs, 4),
		/* AF */ Option::None,
		
		/* B0 */ instr!(bcs, rel, 2),
		/* B1 */ instr!(lda, idy, 5),
		/* B2 */ Option::None,
		/* B3 */ Option::None,
		/* B4 */ Option::None,
		/* B5 */ Option::None,
		/* B6 */ Option::None,
		/* B7 */ Option::None,
		/* B8 */ instr!(clv, imp, 2),
		/* B9 */ Option::None,
		/* BA */ instr!(tsx, imp, 2),
		/* BB */ Option::None,
		/* BC */ Option::None,
		/* BD */ Option::None,
		/* BE */ Option::None,
		/* BF */ Option::None,

		/* C0 */ instr!(cpy, imm, 2),
		/* C1 */ instr!(cmp, idx, 6),
		/* C2 */ Option::None,
		/* C3 */ Option::None,
		/* C4 */ instr!(cpy, zpg, 3),
		/* C5 */ instr!(cmp, zpg, 3),
		/* C6 */ instr!(dec, zpg, 5),
		/* C7 */ Option::None,
		/* C8 */ instr!(iny, imp, 2),
		/* C9 */ instr!(cmp, imm, 2),
		/* CA */ instr!(dex, imp, 2),
		/* CB */ Option::None,
		/* CC */ instr!(cpy, abs, 4),
		/* CD */ instr!(cmp, abs, 4),
		/* CE */ instr!(dec, abs, 6),
		/* CF */ Option::None,

		/* D0 */ instr!(bne, rel, 2),
		/* D1 */ Option::None,
		/* D2 */ Option::None,
		/* D3 */ Option::None,
		/* D4 */ Option::None,
		/* D5 */ Option::None,
		/* D6 */ Option::None,
		/* D7 */ Option::None,
		/* D8 */ instr!(cld, imp, 2),
		/* D9 */ Option::None,
		/* DA */ Option::None,
		/* DB */ Option::None,
		/* DC */ Option::None,
		/* DD */ Option::None,
		/* DE */ Option::None,
		/* DF */ Option::None,

		/* E0 */ instr!(cpx, imm, 2),
		/* E1 */ instr!(sbc, idx, 6),
		/* E2 */ Option::None,
		/* E3 */ Option::None,
		/* E4 */ instr!(cpx, zpg, 3),
		/* E5 */ instr!(sbc, zpg, 3),
		/* E6 */ instr!(inc, zpg, 5),
		/* E7 */ Option::None,
		/* E8 */ instr!(inx, imp, 2),
		/* E9 */ instr!(sbc, imm, 2),
		/* EA */ instr!(nop, imp, 2),
		/* EB */ Option::None,
		/* EC */ instr!(cpx, abs, 4),
		/* ED */ instr!(sbc, abs, 4),
		/* EE */ instr!(inc, abs, 6),
		/* EF */ Option::None,

		/* F0 */ instr!(beq, rel, 2),
		/* F1 */ Option::None,
		/* F2 */ Option::None,
		/* F3 */ Option::None,
		/* F4 */ Option::None,
		/* F5 */ Option::None,
		/* F6 */ Option::None,
		/* F7 */ Option::None,
		/* F8 */ instr!(sed, imp, 2),
		/* F9 */ Option::None,
		/* FA */ Option::None,
		/* FB */ Option::None,
		/* FC */ Option::None,
		/* FD */ Option::None,
		/* FE */ Option::None,
		/* FF */ Option::None
];