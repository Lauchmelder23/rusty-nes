use crate::cpu::CPU;
use crate::bus::Bus;
use std::cell::Ref;

#[allow(dead_code)]
enum Bit
{
	Negative 	= 7,
	Overflow 	= 6,
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

impl CPU 
{
	fn fetch(&mut self) -> u8
	{
		let bus = self.bus.upgrade().unwrap();
		return bus.borrow().read_cpu(self.absolute_addr);
	}

	branch_on_fn!(bcc, Bit::Carry, false);
	branch_on_fn!(bcs, Bit::Carry, true);
	branch_on_fn!(bne, Bit::Zero, false);
	branch_on_fn!(beq, Bit::Zero, true);
	branch_on_fn!(bpl, Bit::Negative, false);
	branch_on_fn!(bmi, Bit::Negative, true);
	branch_on_fn!(bvc, Bit::Overflow, false);
	branch_on_fn!(bvs, Bit::Overflow, true);

	set_flag_fn!(clc, Bit::Carry, false);
	set_flag_fn!(sec, Bit::Carry, true);

	load_fn!(lda, acc);
	load_fn!(ldx, x);
	load_fn!(ldy, y);

	store_fn!(sta, acc);
	store_fn!(stx, x);
	store_fn!(sty, y);

	pub fn bit(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();
		let value = bus.borrow().read_cpu(self.absolute_addr);

		set_flag_to!(self.p, Bit::Negative, (value >> 7) & 0x1);
		set_flag_to!(self.p, Bit::Overflow, (value >> 6) & 0x1);
		set_flag_to!(self.p, Bit::Zero, (self.acc & value) == 0);
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

	pub fn rts(&mut self)
	{
		let bus = self.bus.upgrade().unwrap();

		let lo = pop(bus.borrow(), &mut self.sp) as u16;
		let hi = pop(bus.borrow(), &mut self.sp) as u16;

		self.pc = (hi << 8) | lo;
		self.pc += 1;
	}
}