#[derive(Debug)]
pub struct Cpu{
	//Program counter
	reg_pc: u16,
	//Stack pointer
	reg_sp: u16,

	regs_af: Register, //AF Register
	regs_bc: Register, //BC Register
	regs_de: Register, //DE Register
	regs_hl: Register, //HL Register

}

//TODO Model register F

//struct register to model 8bits registers being paired
#[derive(Debug)]
struct Register{
	//i.e HL Reg => H=Hi, L=Lo
	reg_hi: u8, 
	reg_lo: u8,
}

//TODO FLAG REGISTER

impl Register{

	//new register initialized with num
	fn new(num: u16) -> Self{
		Register{reg_hi:num as u8,reg_lo: (num>>8) as u8}
	}

	//set a value to a pair of registers
	fn set(&mut self,num: u16){
		self.reg_lo=num as u8;
		self.reg_hi=(num>>8) as u8;
	}

	//get the value of the registers as a pair. 
	fn get(&self) -> u16{
		((self.reg_hi as u16) << 8) | (self.reg_lo as u16)
	}

}

impl Cpu{

	//TODO make the initial states a constant??
	//initial state taken from codeslinger
	pub fn new() -> Self{
		Cpu{
			reg_pc: 0x100,
			reg_sp: 0xFFFE,
			regs_af: Register::new(0x01B0),
			regs_bc: Register::new(0x0013),
			regs_de: Register::new(0x00D8),
			regs_hl: Register::new(0x014D)
		}
	}

	/*fn execute_instruction(&self, ){

	}*/
}
