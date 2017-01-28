use gb::interconnect::*;
use gb::opcode::Opcode;
use num::FromPrimitive;
use gb::register::Register;

//FINISH TESTS!!!

#[derive(Debug)]
pub struct Cpu{
	//Program counter
	reg_pc: u16,
	//Stack pointer
	reg_sp: Register,

	regs_af: Register, //AF Register
	regs_bc: Register, //BC Register
	regs_de: Register, //DE Register
	regs_hl: Register, //HL Register

	flags: Flags,

}

#[derive(Debug)]
struct Flags{
	z: bool,
	n: bool,
	h: bool,
	c: bool
}

//Get rid of all the sets? Good or bad practice? DOIT!
impl Cpu{

	//set zero flag
	fn set_zero_flag(&mut self,bit: bool){
		self.flags.z=bit;
	}
	//set subtract flag
	fn set_subtract_flag(&mut self,bit: bool){
		self.flags.n=bit;
	}
	//set half carry flag
	fn set_half_carry_flag(&mut self,bit: bool){
		self.flags.h=bit;
	}
	//set carry flag
	fn set_carry_flag(&mut self,bit: bool){
		self.flags.c=bit;
	}

	//set regs
	fn set_reg_a(&mut self,num: u8){
		self.regs_af.set_hi(num);
	}

	fn set_reg_b(&mut self,num: u8){
		self.regs_bc.set_hi(num);
	}

	fn set_reg_c(&mut self,num: u8){
		self.regs_bc.set_lo(num);
	}

	fn set_reg_d(&mut self,num: u8){
		self.regs_de.set_hi(num);
	}

	fn set_reg_e(&mut self,num: u8){
		self.regs_de.set_lo(num);
	}

	fn set_reg_h(&mut self,num: u8){
		self.regs_hl.set_hi(num);
	}

	fn set_reg_l(&mut self,num: u8){
		self.regs_hl.set_lo(num);
	}

	//gets
	fn get_reg_a(&mut self)->u8{
		self.regs_af.get_hi()
	}

	fn get_reg_b(&mut self)->u8{
		self.regs_bc.get_hi()
	}

	fn get_reg_c(&mut self)->u8{
		self.regs_bc.get_lo()
	}

	fn get_reg_d(&mut self)->u8{
		self.regs_de.get_hi()
	}

	fn get_reg_e(&mut self)->u8{
		self.regs_de.get_lo()
	}

	fn get_reg_h(&mut self)->u8{
		self.regs_hl.get_hi()
	}

	fn get_reg_l(&mut self)->u8{
		self.regs_hl.get_lo()
	}

	//initial state taken from codeslinger (as did almost everything that sounds tricky)
	pub fn new() -> Self{
		Cpu{
			reg_pc: 0x0100,
			reg_sp: Register::new(0xFFFE),
			regs_af: Register::new(0x01B0),
			regs_bc: Register::new(0x0013),
			regs_de: Register::new(0x00D8),
			regs_hl: Register::new(0x014D),
			flags: Flags{z:false,n:false,
							h:false,c:false}
		}
	}

	pub fn execute_next_opcode(&mut self,inter:&mut Interconnect)->usize{
		let op = inter.read(self.reg_pc);
		//wrapping add to prevent overflow
		self.reg_pc=self.reg_pc.wrapping_add(1);
		self.execute_opcode(inter,op)
	}

	//TODO move instructions to separate functions? Will it look better?
	fn execute_opcode(&mut self,inter:&mut Interconnect,opcode: u8) -> usize{
		//unwraps opcode and panics if none
		let value=Opcode::from_u8(opcode).unwrap_or_else(||
            panic!("Unrecognized Opcode: {:#X})",opcode)
        );
        println!("Currently running opcode {:#X} from address {:#X}",opcode,self.reg_pc-1);
        //TODO: Look for a way to remove the Opcode::opcode
		match value{
			//0x00
			Opcode::Nop     => 4,
			//0x0B
			Opcode::Dec_Bc  => {
				////////////////////////////// !!!!!Unwrapped sub???! 
				let num=self.regs_bc.get().wrapping_sub(1);
				self.regs_bc.set(num);
				8
			},
			//TODO:0x25
			Opcode::Dec_H   => {
				4
			},
			//0x2F
			Opcode::Cpl     => {
				let num=self.get_reg_a();
				self.set_reg_a(!num);
				self.set_subtract_flag(true);
				self.set_half_carry_flag(true);
				4
			},
			//0xC3
			Opcode::Jp_a16  => {
				self.reg_pc=inter.read_16bits(self.reg_pc);
				16
			},
			//0x48
			Opcode::Ld_C_B  => {
				let num=self.get_reg_b();
				self.set_reg_c(num);
				4
			},
			//0x49 Is it there for completeness?
			Opcode::Ld_C_C  => 4,
			//0x4A
			Opcode::Ld_C_D  => {
				let num=self.get_reg_d();
				self.set_reg_c(num);
				4
			}
			_ => 0,
		}
	}

	//fn dec
	/*fn get_reg_l(&mut self)->u8{
		self.regs_hl.get_lo()
	}*/
}
