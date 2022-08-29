use std::fmt;

#[derive(Copy, Clone)]
pub struct Mnemonic {
	buf: [char; 4]
}

impl Mnemonic
{
	pub const fn new(content: &str, illegal: bool) -> Mnemonic
	{	
		let mut buf: [char; 4] = [' '; 4];
		if illegal {
			buf[0] = '*';
		} else {
			buf[0] = ' ';
		}

		buf[1] = content.as_bytes()[0] as char;
		buf[2] = content.as_bytes()[1] as char;
		buf[3] = content.as_bytes()[2] as char;

		Mnemonic {
			buf: buf
		}
	} 
}

impl fmt::Display for Mnemonic 
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
	{
		write!(f, "{}{}{}{}", self.buf[0], self.buf[1], self.buf[2], self.buf[3])
	}
}