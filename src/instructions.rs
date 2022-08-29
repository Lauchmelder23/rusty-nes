use crate::cpu::{CPU, FetchType};
use crate::bus::Bus;
use std::cell::Ref;

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

	pub fn adc(&mut self)
	{
		let value = self.fetch() as u16;
		let result = (self.acc as u16) + value + (test_flag!(self.p, Bit::Carry) as u16);

		set_flag_to!(self.p, Bit::Carry, (result & 0xFF00) != 0x0000);
		set_flag_to!(self.p, Bit::Negative, ((result >> 7) & 0x0001) == 0x0001);
		set_flag_to!(self.p, Bit::Zero, (result & 0x00FF) == 0x0000);
		set_flag_to!(self.p, Bit::Overflow, ((result ^ value) & (result ^ self.acc as u16) & 0x80) == 0x80);

		self.acc = result as u8;
	}

	pub fn sbc(&mut self)
	{
		let value = !(self.fetch() as u16);
		let result = (self.acc as u16).wrapping_add(value).wrapping_add(test_flag!(self.p, Bit::Carry) as u16);

		set_flag_to!(self.p, Bit::Carry, (result & 0xFF00) == 0x0000);
		set_flag_to!(self.p, Bit::Negative, ((result >> 7) & 0x0001) == 0x0001);
		set_flag_to!(self.p, Bit::Zero, (result & 0x00FF) == 0x0000);
		set_flag_to!(self.p, Bit::Overflow, ((result ^ value) & (result ^ self.acc as u16) & 0x80) == 0x80);

		self.acc = result as u8;
	}

	pub fn and(&mut self)
	{
		let val = self.fetch();

		self.acc &= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	pub fn bit(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		let value = bus.borrow().read_cpu(self.absolute_addr);

		set_flag_to!(self.p, Bit::Negative, (value >> 7) & 0x1);
		set_flag_to!(self.p, Bit::Overflow, (value >> 6) & 0x1);
		set_flag_to!(self.p, Bit::Zero, (self.acc & value) == 0);
	}

	pub fn eor(&mut self)
	{
		let val = self.fetch();

		self.acc ^= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	pub fn jmp(&mut self)
	{
		self.pc = self.absolute_addr;
	}

	pub fn jsr(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.pc -= 1;
		push!(bus.borrow_mut(), self.sp, self.pc >> 8);
		push!(bus.borrow_mut(), self.sp, self.pc);

		self.pc = self.absolute_addr;
	}

	pub fn nop(&mut self)
	{
		
	}

	pub fn ora(&mut self)
	{
		let val = self.fetch();

		self.acc |= val;
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	pub fn pha(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		push!(bus.borrow_mut(), self.sp, self.acc);
	}

	pub fn pla(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		self.acc = pop(bus.borrow(), &mut self.sp);
		set_flag_to!(self.p, Bit::Negative, (self.acc & (1u8 << 7)) > 0);
		set_flag_to!(self.p, Bit::Zero, self.acc == 0);
	}

	pub fn php(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let mut value = self.p;
		set_flag!(value, Bit::Break);
		set_flag!(value, 5);

		push!(bus.borrow_mut(), self.sp, value);
	}

	pub fn plp(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let flag: u8 = pop(bus.borrow(), &mut self.sp);
		let mask: u8 = 0b11001111;

		self.p &= !mask;
		self.p |= flag & mask;
	}

	pub fn rti(&mut self)
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

	pub fn rts(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let lo = pop(bus.borrow(), &mut self.sp) as u16;
		let hi = pop(bus.borrow(), &mut self.sp) as u16;

		self.pc = (hi << 8) | lo;
		self.pc += 1;
	}
}