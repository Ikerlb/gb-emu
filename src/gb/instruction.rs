use gb::opcode::Opcode;

struct Instruction{
	opcode: Opcode,
	byte_length: usize,
}