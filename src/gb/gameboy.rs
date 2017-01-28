use gb::cpu::*;
use gb::interconnect::*;
use std::fmt::{Display,Result,Formatter};

pub struct GameBoy{
	cpu: Cpu,
	interconnect: Interconnect,
}

impl GameBoy{

	pub fn new(cart:Vec<u8>)->Self{
		GameBoy{cpu: Cpu::new(),interconnect: Interconnect::new(cart)}
	}

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn interconnect(&self) -> &Interconnect {
        &self.interconnect
    }

    pub fn run(&mut self){
        loop{
            self.cpu.execute_next_opcode(&mut self.interconnect);
            println!("{}",self);
        }
    }

}


//Implementing display trait for debugging purposes.
impl Display for GameBoy {
    fn fmt(&self, f: &mut Formatter) -> Result {
    	write!(f,"{:#?}",self.cpu)
    }
}