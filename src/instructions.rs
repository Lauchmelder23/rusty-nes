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
	($instr: ident, $addr: ident, $cyc: literal, $illegal: literal) =>
	{
		Option::Some(Instruction 
		{
			action: CPU::$instr,
			addressing: CPU::$addr,
			cycles: $cyc,
			length: instr_size!($addr),

			name: Mnemonic::new(stringify!($instr), $illegal)
		})
	};

	($instr: ident, $addr: ident, $cyc: literal) => { instr!($instr, $addr, $cyc, false) };
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
			
		$self.additional_cycles += 1;
		if (branch_target & 0xFF00) != ($self.pc & 0xFF00)	// Branched to different page
		{
			$self.additional_cycles += 1;
		}

		$self.pc = branch_target;
	}
}

macro_rules! branch_on_fn
{
	($name: ident, $flag: expr, $result: literal) => 
	{
		fn $name(&mut self)
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
		fn $name(&mut self)
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
		fn $name(&mut self) 
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
		fn $name(&mut self) 
		{
			let bus = self.bus.upgrade().unwrap();
			bus.borrow_mut().write_cpu(self.absolute_addr, self.$register);

			self.additional_cycles = 0;
		}
	};
}

macro_rules! transfer_fn
{
	($name: ident, $from: ident, $to: ident) => 
	{
		fn $name(&mut self)
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
		fn $name(&mut self)
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
		fn $name(&mut self)
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
		fn $name(&mut self)
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
		fn $name(&mut self)
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

macro_rules! invoke_functions
{
	($self: ident, $func: ident) => ($self.$func());

	($self: ident, $func: ident, $($next: ident),+) => (
		$self.$func();
		invoke_functions!($self, $($next),+);
	)
}

macro_rules! combine_instructions
{
	($name: ident, no_additional_cycles, $($parts: ident),+) => 
	{
		fn $name(&mut self)
		{
			invoke_functions!(self, $($parts),+);
			self.additional_cycles = 0;
		}
	};

	($name: ident, $($parts: ident),+) => 
	{
		fn $name(&mut self)
		{
			invoke_functions!(self, $($parts),+);
		}
	};
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

	fn eor(&mut self)
	{
		let val = self.fetch();

		self.acc ^= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	fn ora(&mut self)
	{
		let val = self.fetch();

		self.acc |= val;
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

	fn brk(&mut self) 
	{
		let bus = self.bus.upgrade().unwrap();
	}

	///// ILLEGAL OPCODES
	
	combine_instructions!(dcp, no_additional_cycles, dec, cmp);
	combine_instructions!(lax, lda, ldx);
	combine_instructions!(isc, no_additional_cycles, inc, sbc);
	combine_instructions!(slo, no_additional_cycles, asl, ora);
	combine_instructions!(rla, no_additional_cycles, rol, and);
	combine_instructions!(sre, no_additional_cycles, lsr, eor);
	combine_instructions!(rra, no_additional_cycles, ror, adc);

	fn sax(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		bus.borrow_mut().write_cpu(self.absolute_addr, self.acc & self.x);

		self.additional_cycles = 0;
	}
}


pub static INSTRUCTION_SET: [Option<Instruction>; 256] = [
		/* 00 */ Option::None, //instr!(brk, imp, 7),
		/* 01 */ instr!(ora, idx, 6),
		/* 02 */ Option::None,
		/* 03 */ instr!(slo, idx, 8, true),
		/* 04 */ instr!(nop, zpg, 3, true),
		/* 05 */ instr!(ora, zpg, 3),
		/* 06 */ instr!(asl, zpg, 5),
		/* 07 */ instr!(slo, zpg, 5, true),
		/* 08 */ instr!(php, imp, 3),
		/* 09 */ instr!(ora, imm, 2),
		/* 0A */ instr!(asl, acc, 2),
		/* 0B */ Option::None,
		/* 0C */ instr!(nop, abs, 4, true),
		/* 0D */ instr!(ora, abs, 4),
		/* 0E */ instr!(asl, abs, 6),
		/* 0F */ instr!(slo, abs, 6, true),

		/* 10 */ instr!(bpl, rel, 2),
		/* 11 */ instr!(ora, idy, 5),
		/* 12 */ Option::None,
		/* 13 */ instr!(slo, idy, 8, true),
		/* 14 */ instr!(nop, zpx, 4, true),
		/* 15 */ instr!(ora, zpx, 4),
		/* 16 */ instr!(asl, zpx, 6),
		/* 17 */ instr!(slo, zpx, 6, true),
		/* 18 */ instr!(clc, imp, 2),
		/* 19 */ instr!(ora, aby, 4),
		/* 1A */ instr!(nop, imp, 2, true),
		/* 1B */ instr!(slo, aby, 7, true),
		/* 1C */ instr!(nop, abx, 4, true),
		/* 1D */ instr!(ora, abx, 4),
		/* 1E */ instr!(asl, abx, 7),
		/* 1F */ instr!(slo, abx, 7, true),

		/* 20 */ instr!(jsr, abs, 6),
		/* 21 */ instr!(and, idx, 6),
		/* 22 */ Option::None,
		/* 23 */ instr!(rla, idx, 8, true),
		/* 24 */ instr!(bit, zpg, 3),
		/* 25 */ instr!(and, zpg, 3),
		/* 26 */ instr!(rol, zpg, 5),
		/* 27 */ instr!(rla, zpg, 5, true),
		/* 28 */ instr!(plp, imp, 4),
		/* 29 */ instr!(and, imm, 2),
		/* 2A */ instr!(rol, acc, 2),
		/* 2B */ Option::None,
		/* 2C */ instr!(bit, abs, 4),
		/* 2D */ instr!(and, abs, 4),
		/* 2E */ instr!(rol, abs, 6),
		/* 2F */ instr!(rla, abs, 6, true),

		/* 30 */ instr!(bmi, rel, 2),
		/* 31 */ instr!(and, idy, 5),
		/* 32 */ Option::None,
		/* 33 */ instr!(rla, idy, 8, true),
		/* 34 */ instr!(nop, zpx, 4, true),
		/* 35 */ instr!(and, zpx, 4),
		/* 36 */ instr!(rol, zpx, 6),
		/* 37 */ instr!(rla, zpx, 6, true),
		/* 38 */ instr!(sec, imp, 2),
		/* 39 */ instr!(and, aby, 4),
		/* 3A */ instr!(nop, imp, 2, true),
		/* 3B */ instr!(rla, aby, 7, true),
		/* 3C */ instr!(nop, abx, 4, true),
		/* 3D */ instr!(and, abx, 4),
		/* 3E */ instr!(rol, abx, 7),
		/* 3F */ instr!(rla, abx, 7, true),

		/* 40 */ instr!(rti, imp, 6),
		/* 41 */ instr!(eor, idx, 6),
		/* 42 */ Option::None,
		/* 43 */ instr!(sre, idx, 8, true),
		/* 44 */ instr!(nop, zpg, 3, true),
		/* 45 */ instr!(eor, zpg, 3),
		/* 46 */ instr!(lsr, zpg, 5),
		/* 47 */ instr!(sre, zpg, 5, true),
		/* 48 */ instr!(pha, imp, 3),
		/* 49 */ instr!(eor, imm, 2),
		/* 4A*/  instr!(lsr, acc, 2),
		/* 4B */ Option::None,
		/* 4C */ instr!(jmp, abs, 3),
		/* 4D */ instr!(eor, abs, 4),
		/* 4E */ instr!(lsr, abs, 6),
		/* 4F */ instr!(sre, abs, 6, true),

		/* 50 */ instr!(bvc, rel, 2),
		/* 51 */ instr!(eor, idy, 5),
		/* 52 */ Option::None,
		/* 53 */ instr!(sre, idy, 8, true),
		/* 54 */ instr!(nop, zpx, 4, true),
		/* 55 */ instr!(eor, zpx, 4),
		/* 56 */ instr!(lsr, zpx, 6),
		/* 57 */ instr!(sre, zpx, 6, true),
		/* 58 */ instr!(sre, aby, 7, true),
		/* 59 */ instr!(eor, aby, 4),
		/* 5A */ instr!(nop, imp, 2, true),
		/* 5B */ instr!(sre, aby, 7, true),
		/* 5C */ instr!(nop, abx, 4, true),
		/* 5D */ instr!(eor, abx, 4),
		/* 5E */ instr!(lsr, abx, 7),
		/* 5F */ instr!(sre, abx, 7, true),

		/* 60 */ instr!(rts, imp, 6),
		/* 61 */ instr!(adc, idx, 6),
		/* 62 */ Option::None,
		/* 63 */ instr!(rra, idx, 8, true),
		/* 64 */ instr!(nop, zpg, 3, true),
		/* 65 */ instr!(adc, zpg, 3),
		/* 66 */ instr!(ror, zpg, 5),
		/* 67 */ instr!(rra, zpg, 5, true),
		/* 68 */ instr!(pla, imp, 4),
		/* 69 */ instr!(adc, imm, 2),
		/* 6A */ instr!(ror, acc, 2),
		/* 6B */ Option::None,
		/* 6C */ instr!(jmp, ind, 5),
		/* 6D */ instr!(adc, abs, 4),
		/* 6E */ instr!(ror, abs, 6),
		/* 6F */ instr!(rra, abs, 6, true),
		
		/* 70 */ instr!(bvs, rel, 2),
		/* 71 */ instr!(adc, idy, 5),
		/* 72 */ Option::None,
		/* 73 */ instr!(rra, idy, 8, true),
		/* 74 */ instr!(nop, zpx, 4, true),
		/* 75 */ instr!(adc, zpx, 4),
		/* 76 */ instr!(ror, zpx, 6),
		/* 77 */ instr!(rra, zpx, 6, true),
		/* 78 */ instr!(sei, imp, 2),
		/* 79 */ instr!(adc, aby, 4),
		/* 7A */ instr!(nop, imp, 2, true),
		/* 7B */ instr!(rra, aby, 7, true),
		/* 7C */ instr!(nop, abx, 4, true),
		/* 7D */ instr!(adc, abx, 4),
		/* 7E */ instr!(ror, abx, 7),
		/* 7F */ instr!(rra, abx, 7, true),

		/* 80 */ instr!(nop, imm, 2, true),
		/* 81 */ instr!(sta, idx, 6),
		/* 82 */ instr!(nop, imm, 2, true),
		/* 83 */ instr!(sax, idx, 6, true),
		/* 84 */ instr!(sty, zpg, 3),
		/* 85 */ instr!(sta, zpg, 3),
		/* 86 */ instr!(stx, zpg, 3),
		/* 87 */ instr!(sax, zpg, 3, true),
		/* 88 */ instr!(dey, imp, 2),
		/* 89 */ instr!(nop, imm, 2, true),
		/* 8A */ instr!(txa, imp, 2),
		/* 8B */ Option::None,
		/* 8C */ instr!(sty, abs, 4),
		/* 8D */ instr!(sta, abs, 4),
		/* 8E */ instr!(stx, abs, 4),
		/* 8F */ instr!(sax, abs, 4, true),

		/* 90 */ instr!(bcc, rel, 2),
		/* 91 */ instr!(sta, idy, 6),
		/* 92 */ Option::None,
		/* 93 */ Option::None,
		/* 94 */ instr!(sty, zpx, 4),
		/* 95 */ instr!(sta, zpx, 4),
		/* 96 */ instr!(stx, zpy, 4),
		/* 97 */ instr!(sax, zpy, 4, true),
		/* 98 */ instr!(tya, imp, 2),
		/* 99 */ instr!(sta, aby, 5),
		/* 9A */ instr!(txs, imp, 2),
		/* 9B */ Option::None,
		/* 9C */ Option::None,
		/* 9D */ instr!(sta, abx, 5),
		/* 9E */ Option::None,
		/* 9F */ Option::None,

		/* A0 */ instr!(ldy, imm, 2),
		/* A1 */ instr!(lda, idx, 6),
		/* A2 */ instr!(ldx, imm, 2),
		/* A3 */ instr!(lax, idx, 6, true),
		/* A4 */ instr!(ldy, zpg, 3),
		/* A5 */ instr!(lda, zpg, 3),
		/* A6 */ instr!(ldx, zpg, 3),
		/* A7 */ instr!(lax, zpg, 3, true),
		/* A8 */ instr!(tay, imp, 2),
		/* A9 */ instr!(lda, imm, 2),
		/* AA */ instr!(tax, imp, 2),
		/* AB */ Option::None,
		/* AC */ instr!(ldy, abs, 4),
		/* AD */ instr!(lda, abs, 4),
		/* AE */ instr!(ldx, abs, 4),
		/* AF */ instr!(lax, abs, 4, true),
		
		/* B0 */ instr!(bcs, rel, 2),
		/* B1 */ instr!(lda, idy, 5),
		/* B2 */ Option::None,
		/* B3 */ instr!(lax, idy, 5, true),
		/* B4 */ instr!(ldy, zpx, 4),
		/* B5 */ instr!(lda, zpx, 4),
		/* B6 */ instr!(ldx, zpy, 4),
		/* B7 */ instr!(lax, zpy, 4, true),
		/* B8 */ instr!(clv, imp, 2),
		/* B9 */ instr!(lda, aby, 4),
		/* BA */ instr!(tsx, imp, 2),
		/* BB */ Option::None,
		/* BC */ instr!(ldy, abx, 4),
		/* BD */ instr!(lda, abx, 4),
		/* BE */ instr!(ldx, aby, 4),
		/* BF */ instr!(lax, aby, 4, true),

		/* C0 */ instr!(cpy, imm, 2),
		/* C1 */ instr!(cmp, idx, 6),
		/* C2 */ instr!(nop, imm, 2, true),
		/* C3 */ instr!(dcp, idx, 8, true),
		/* C4 */ instr!(cpy, zpg, 3),
		/* C5 */ instr!(cmp, zpg, 3),
		/* C6 */ instr!(dec, zpg, 5),
		/* C7 */ instr!(dcp, zpg, 5, true),
		/* C8 */ instr!(iny, imp, 2),
		/* C9 */ instr!(cmp, imm, 2),
		/* CA */ instr!(dex, imp, 2),
		/* CB */ Option::None,
		/* CC */ instr!(cpy, abs, 4),
		/* CD */ instr!(cmp, abs, 4),
		/* CE */ instr!(dec, abs, 6),
		/* CF */ instr!(dcp, abs, 6, true),

		/* D0 */ instr!(bne, rel, 2),
		/* D1 */ instr!(cmp, idy, 5),
		/* D2 */ Option::None,
		/* D3 */ instr!(dcp, idy, 8, true),
		/* D4 */ instr!(nop, zpx, 4, true),
		/* D5 */ instr!(cmp, zpx, 4),
		/* D6 */ instr!(dec, zpx, 6),
		/* D7 */ instr!(dcp, zpx, 6, true),
		/* D8 */ instr!(cld, imp, 2),
		/* D9 */ instr!(cmp, aby, 4),
		/* DA */ instr!(nop, imp, 2, true),
		/* DB */ instr!(dcp, aby, 7, true),
		/* DC */ instr!(nop, abx, 4, true),
		/* DD */ instr!(cmp, abx, 4),
		/* DE */ instr!(dec, abx, 7),
		/* DF */ instr!(dcp, abx, 7, true),

		/* E0 */ instr!(cpx, imm, 2),
		/* E1 */ instr!(sbc, idx, 6),
		/* E2 */ instr!(nop, imm, 2, true),
		/* E3 */ instr!(isc, idx, 8, true),
		/* E4 */ instr!(cpx, zpg, 3),
		/* E5 */ instr!(sbc, zpg, 3),
		/* E6 */ instr!(inc, zpg, 5),
		/* E7 */ instr!(isc, zpg, 5, true),
		/* E8 */ instr!(inx, imp, 2),
		/* E9 */ instr!(sbc, imm, 2),
		/* EA */ instr!(nop, imp, 2),
		/* EB */ instr!(sbc, imm, 2, true),
		/* EC */ instr!(cpx, abs, 4),
		/* ED */ instr!(sbc, abs, 4),
		/* EE */ instr!(inc, abs, 6),
		/* EF */ instr!(isc, abs, 6, true),

		/* F0 */ instr!(beq, rel, 2),
		/* F1 */ instr!(sbc, idy, 5),
		/* F2 */ Option::None,
		/* F3 */ instr!(isc, idy, 8, true),
		/* F4 */ instr!(nop, zpx, 4, true),
		/* F5 */ instr!(sbc, zpx, 4),
		/* F6 */ instr!(inc, zpx, 6),
		/* F7 */ instr!(isc, zpx, 6, true),
		/* F8 */ instr!(sed, imp, 2),
		/* F9 */ instr!(sbc, aby, 4),
		/* FA */ instr!(nop, imp, 2, true),
		/* FB */ instr!(isc, aby, 7, true),
		/* FC */ instr!(nop, abx, 4, true),
		/* FD */ instr!(sbc, abx, 4),
		/* FE */ instr!(inc, abx, 7),
		/* FF */ instr!(isc, abx, 7, true),
];