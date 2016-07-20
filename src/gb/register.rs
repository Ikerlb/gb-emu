//struct register to model 8bits registers being paired
#[derive(Debug)]
pub struct Register{
	//i.e HL Reg => H=Hi, L=Lo
	hi: u8, 
	lo: u8,
}

//struct to model reg AF
#[derive(Debug)]
pub struct RegisterAF{
	hi: u8, //Accumulator
	lo: u8, //Flag
}

//todo trait?
impl RegisterAF{
	//new register initialized with num.
	pub fn new(num: u16) -> Self{
		RegisterAF{hi:num as u8,lo: (num>>8) as u8}
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

	pub fn enable_zero_flag(&mut self){
		self.hi|=0b10000000;
	}

	pub fn enable_subtract_flag(&mut self){
		self.hi|=0b01000000;
	}

	pub fn enable_half_carry_flag(&mut self){
		self.hi|=0b00100000;
	}

	pub fn enable_carry_flag(&mut self){
		self.hi|=0b00010000;
	}	

	//is zero flag on?
	pub fn zero_flag(&self)->bool{
		self.hi&0b10000000!=0	
	}

	//is subtract flag on?
	pub fn subtract_flag(&self)->bool{
		self.hi&0b01000000!=0
	}

	//is half carry flag on?
	pub fn half_carry_flag(&self)->bool{
		self.hi&0b00100000!=0
	}

	//is carry flag on?
	pub fn carry_flag(&self)->bool{
		self.hi&0b00010000!=0
	}
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

}
#[test]
fn enable_flags(){
	let mut reg=RegisterAF::new(0);
	assert!(!reg.zero_flag());
	assert!(!reg.subtract_flag());
	assert!(!reg.half_carry_flag());
	assert!(!reg.carry_flag());
	reg.enable_zero_flag();
	reg.enable_subtract_flag();
	reg.enable_half_carry_flag();
	reg.enable_carry_flag();
	assert_eq!(reg.hi,0b11110000);
	assert!(reg.zero_flag());
	assert!(reg.subtract_flag());
	assert!(reg.half_carry_flag());
	assert!(reg.carry_flag());
}

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