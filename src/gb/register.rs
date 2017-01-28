//TODO make attributes public????????????
//struct register to model 8bits registers being paired
#[derive(Debug)]
pub struct Register{
	//i.e HL Reg => H=Hi, L=Lo
	hi: u8, 
	lo: u8,
}

impl Register{

	//new register initialized with num.
	pub fn new(num: u16) -> Self{
		Register{hi:num as u8,lo: (num>>8) as u8}
	}

	//set a value to a pair of registers.
	pub fn set(&mut self,num: u16){
		self.lo=num as u8;
		self.hi=(num>>8) as u8;
	}

	//get the value of the registers as a pair. 
	pub fn get(&self) -> u16{
		((self.hi as u16) << 8) | (self.lo as u16)
	}

	pub fn set_hi(&mut self,num: u8){
		self.hi=num;
	}

	pub fn set_lo(&mut self,num:u8){
		self.lo=num;
	}

	pub fn get_hi(&self) -> u8{
		self.hi
	}

	pub fn get_lo(&self) -> u8{
		self.lo
	}
}

#[cfg(test)]
mod tests{

	use super::Register;

	#[test]
	fn set_register(){
		let mut reg = Register::new(0xFFFF);
		reg.set(0x00FF);
		assert_eq!(reg.hi,0x00u8);
		assert_eq!(reg.lo,0xFFu8);	
	}

	#[test]
	fn get_register(){
		let mut reg=Register::new(0xFFFF);
		reg.hi = 0x00;
		assert_eq!(reg.get(),0x00FFu16);
	}

	#[test]
	fn set_hi(){
		let mut reg=Register::new(0);
		reg.hi=0xFF;
		assert_eq!(reg.hi,0xFFu8);
	}

	#[test]
	fn set_lo(){
		let mut reg=Register::new(0);
		reg.lo=0xFF;
		assert_eq!(reg.lo,0xFF);
	}
}