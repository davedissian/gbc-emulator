use std::rc::Rc;
use std::cell::RefCell;
use memory::Memory;
use cpu::{Cond, IndirectAddr};
use cpu::registers::*;
use cpu::ops::*;
use cpu::fetcher::*;

// CPU Data
pub struct Cpu {
    pub running: bool,
    memory: Rc<RefCell<Memory>>,
    regs: Registers
}

// Registers
macro_rules! read_reg_pair {
    ($h:expr, $l:expr) => {
        (($h as u16) << 8) | $l as u16
    };
}

macro_rules! write_reg_pair {
    ($h:expr, $l:expr, $v:expr) => {{
        $h = ($v >> 8) as u8;
        $l = ($v & 0xFF) as u8;
    }};
}

fn get_address(cpu: &Cpu, a: &IndirectAddr) -> u16 {
    match *a {
        IndirectAddr::BC => read_reg_pair!(cpu.regs.b, cpu.regs.c),
        IndirectAddr::DE => read_reg_pair!(cpu.regs.d, cpu.regs.e),
        IndirectAddr::HL => read_reg_pair!(cpu.regs.h, cpu.regs.l),
        IndirectAddr::C => cpu.regs.c as u16 + 0xFF00,
        IndirectAddr::Imm8(n) => n as u16 + 0xFF00,
        IndirectAddr::Imm16(n) => n
    }
}

impl Fetcher for Cpu {
    fn fetch_word(&mut self) -> u8 {
        let byte = self.mem_read_u8(self.regs.pc);
        self.regs.pc += 1;
        byte
    }
}
    
// Helper function to get a single bit
fn get_flag_bit(value: u16, bit: u8) -> bool {
    ((value >> bit) & 0x1) == 1
}

// Interpreter implementation of the CPU ops defined in the ops module
#[allow(unused_variables)]
impl<'a> CpuOps for &'a mut Cpu {
    fn read_arg8(&self, arg: Arg8) -> u8 {
        match arg {
            Arg8::Reg(r) => match r {
                Reg8::A => self.regs.a,
                Reg8::B => self.regs.b,
                Reg8::C => self.regs.c,
                Reg8::D => self.regs.d,
                Reg8::E => self.regs.e,
                Reg8::F => self.regs.f,
                Reg8::H => self.regs.h,
                Reg8::L => self.regs.l
            },

            Arg8::Ind(addr) => {
                let addr = get_address(self, &addr);
                self.mem_read_u8(addr)
            }

            Arg8::Imm(v) => v
        }
    }

    fn write_arg8(&mut self, arg: Arg8, data: u8) {
        match arg {
            Arg8::Reg(r) => match r {
                Reg8::A => self.regs.a = data,
                Reg8::B => self.regs.b = data,
                Reg8::C => self.regs.c = data,
                Reg8::D => self.regs.d = data,
                Reg8::E => self.regs.e = data,
                Reg8::F => self.regs.f = data,
                Reg8::H => self.regs.h = data,
                Reg8::L => self.regs.l = data
            },

            Arg8::Ind(addr) => {
                let addr = get_address(self, &addr);
                self.mem_write_u8(addr, data);
            },

            _ => panic!("Cannot write to {:?}", arg)
        }
    }

    fn read_arg16(&self, arg: Arg16) -> u16 {
        match arg {
            Arg16::Reg(r) => match r {
                Reg16::AF => read_reg_pair!(self.regs.a, self.regs.f),
                Reg16::BC => read_reg_pair!(self.regs.b, self.regs.c),
                Reg16::DE => read_reg_pair!(self.regs.d, self.regs.e),
                Reg16::HL => read_reg_pair!(self.regs.h, self.regs.l),
                Reg16::SP => self.regs.sp,
                Reg16::PC => self.regs.pc,
            },

            Arg16::Ind(addr) => {
                let addr = get_address(self, &addr);
                self.mem_read_u16(addr)
            },

            Arg16::Imm(v) => v
        }
    }

    fn write_arg16(&mut self, arg: Arg16, data: u16) {
        match arg {
            Arg16::Reg(r) => match r {
                Reg16::AF => write_reg_pair!(self.regs.a, self.regs.f, data),
                Reg16::BC => write_reg_pair!(self.regs.b, self.regs.c, data),
                Reg16::DE => write_reg_pair!(self.regs.d, self.regs.e, data),
                Reg16::HL => write_reg_pair!(self.regs.h, self.regs.l, data),
                Reg16::SP => self.regs.sp = data,
                Reg16::PC => self.regs.pc = data,
            },

            Arg16::Ind(addr) => {
                let addr = get_address(self, &addr);
                self.mem_write_u16(addr, data);
            },

            _ => panic!("Cannot write to {:?}", arg)
        }
    }

    fn ld(&mut self, o: Arg8, i: Arg8) {
        let value = self.read_arg8(i);
        self.write_arg8(o, value);
    }
    
    fn ldd(&mut self, o: Arg8, i: Arg8) {
    }

    fn ldi(&mut self, o: Arg8, i: Arg8) {
    }

    fn ldh(&mut self, o: Arg8, i: Arg8){
    }

    fn ld16(&mut self, o: Arg16, i: Arg16) {
        let value = self.read_arg16(i);
        self.write_arg16(o, value);
    }

    fn ld16_hlsp(&mut self, offset: i8) {
        let value = if offset < 0 {
            self.regs.sp - (offset as u16)
        } else {
            self.regs.sp + (offset as u16)
        };
        write_reg_pair!(self.regs.h, self.regs.l, value);
    }

    // TODO(David): Should the stack pointer be decremented before or after reading from memory?
    fn push(&mut self, i: Arg16) {
        let sp = self.regs.sp;
        let content = self.read_arg16(i);
        self.mem_write_u16(sp, content);
        self.regs.sp -= 2;
    }

    fn pop(&mut self, o: Arg16) {
        self.regs.sp += 2;
        let value = self.mem_read_u16(self.regs.sp);
        self.write_arg16(o, value);
    }

    fn add(&mut self, i: Arg8) {
        let result = self.regs.a as u16 + self.read_arg8(i) as u16;
        self.regs.a = result as u8;
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.update_flag(Flag::H, get_flag_bit(result, 4));
        self.regs.update_flag(Flag::C, get_flag_bit(result, 8));
    }

    fn adc(&mut self, i: Arg8) {
        let result =
            self.regs.a as u16 +
            self.read_arg8(i) as u16 +
            if self.regs.get_flag(Flag::C) { 1 } else { 0 };
        self.regs.a = result as u8;
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.update_flag(Flag::H, get_flag_bit(result, 4));
        self.regs.update_flag(Flag::C, get_flag_bit(result, 8));
    }

    fn sub(&mut self, i: Arg8) {
        let result = self.regs.a as u16 - self.read_arg8(i) as u16;
        self.regs.a = result as u8;

        // TODO(David): Flags
    }

    fn sbc(&mut self, i: Arg8) {
        let result =
            self.regs.a as u16 -
            self.read_arg8(i) as u16 -
            if self.regs.get_flag(Flag::C) { 1 } else { 0 };
        self.regs.a = result as u8;

        // TODO(David): Flags
    }

    fn and(&mut self, i: Arg8) {
        self.regs.a &= self.read_arg8(i);
        let result = self.regs.a;
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.set_flag(Flag::H);
        self.regs.reset_flag(Flag::C);
    }

    fn or(&mut self, i: Arg8) {
        self.regs.a |= self.read_arg8(i);
        let result = self.regs.a;
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
        self.regs.reset_flag(Flag::C);
    }

    fn xor(&mut self, i: Arg8) {
        self.regs.a ^= self.read_arg8(i);
        let result = self.regs.a;
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
        self.regs.reset_flag(Flag::C);
    }

    fn cp(&mut self, i: Arg8) {
        let result = self.regs.a as u16 - self.read_arg8(i) as u16;
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.set_flag(Flag::N);
        // TODO(David): H and C flags
    }

    fn inc(&mut self, io: Arg8) {
        let result = self.read_arg8(io) + 1;
        self.write_arg8(io, result);
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.update_flag(Flag::H, get_flag_bit(result as u16, 3));
    }

    fn dec(&mut self, io: Arg8) {
        let result = self.read_arg8(io) - 1;
        self.write_arg8(io, result);
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.set_flag(Flag::N);
        // TODO(David): H flag
    }

    fn add16(&mut self, i: Arg16) {
        let result =
            read_reg_pair!(self.regs.h, self.regs.l) as u32 +
            self.read_arg16(i) as u32;
        write_reg_pair!(self.regs.h, self.regs.l, result as u16);
        self.regs.reset_flag(Flag::N);
        self.regs.update_flag(Flag::H, get_flag_bit(result as u16, 12));
        self.regs.update_flag(Flag::C, get_flag_bit(result as u16, 16));
    }

    fn add16_sp(&mut self, i: u8) {
        //TODO(Csongor): this was not actually setting
        //the stack pointer anyway, so I've ust commented
        //it out for now

        //let result = self.regs.sp + self.read_arg8(i) as i8;
        //self.regs.reset_flag(Flag::Z);
        //self.regs.reset_flag(Flag::N);
        // TODO(David): H and C flags are ambiguously defined
    }

    fn inc16(&mut self, io: Arg16) {
        let result = self.read_arg16(io) + 1;
        self.write_arg16(io, result);
    }

    fn dec16(&mut self, io: Arg16) {
        let result = self.read_arg16(io) - 1;
        self.write_arg16(io, result);
    }

    // misc
    fn nop(&mut self) {}

    fn daa(&mut self) {
        // TODO(David): Ambiguous spec, test this
        // A stores a number up to 255. In BCD form each nibble would store a single digit,
        // therefore the maximum number that can be stored is 99.

        // Source:
        // The DAA instruction corrects this invalid result. It checks to see if there was a carry
        // out of the low order BCD digit and adjusts the value (by adding six to it) if there was
        // an overflow. After adjusting for overflow out of the L.O. digit, the DAA instruction
        // repeats this process for the H.O. digit. DAA sets the carry flag if the was a (decimal)
        // carry out of the H.O. digit of the operation.
    }

    fn cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.set_flag(Flag::N);
        self.regs.set_flag(Flag::H);
    }

    fn ccf(&mut self) {
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
        let current_flag = self.regs.get_flag(Flag::C);
        self.regs.update_flag(Flag::C, !current_flag);
    }

    fn scf(&mut self) {
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
        self.regs.set_flag(Flag::C);
    }

    fn halt(&mut self) {
    }

    fn stop(&mut self) {
    }

    fn ei(&mut self) {
    }

    fn di(&mut self) {
    }

    // rotate and shift
    fn rlc(&mut self, io: Arg8) {
        let value = self.read_arg8(io);
        self.regs.update_flag(Flag::C, get_flag_bit(value as u16, 7));
        let result = value << 1;
        self.write_arg8(io, result);
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
    }

    fn rl(&mut self, io: Arg8) {
        // TODO(David): Spec is ambiguous again, what's the difference between RL and RLC?
        self.rlc(io);
    }

    fn rrc(&mut self, io: Arg8) {
        let value = self.read_arg8(io);
        self.regs.update_flag(Flag::C, get_flag_bit(value as u16, 0));
        let result = value >> 1;
        self.write_arg8(io, result);
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
    }

    fn rr(&mut self, io: Arg8) {
        // TODO(David): Spec is ambiguous again, what's the difference between RR and RRC?
        self.rrc(io);
    }

    fn sla(&mut self, io: Arg8) {
        let result = (self.read_arg8(io) as u16) << 1;
        self.write_arg8(io, result as u8);
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
        self.regs.update_flag(Flag::C, get_flag_bit(result, 8));
    }

    fn sra(&mut self, io: Arg8) {
        let value = self.read_arg8(io);
        self.regs.update_flag(Flag::C, get_flag_bit(value as u16, 0));
        let result = value >> 1;
        self.write_arg8(io, result);
        self.regs.update_flag(Flag::Z, result == 0);
        self.regs.reset_flag(Flag::N);
        self.regs.reset_flag(Flag::H);
    }

    fn swap(&mut self, io: Arg8) {
        let initial = self.read_arg8(io);
        self.write_arg8(io, ((initial >> 4) & 0xF) | ((initial << 4) & 0xF));
    }

    fn srl(&mut self, io: Arg8) {
    }

    // bit manipulation
    fn bit(&mut self, bit_id: u8, o: Arg8) {
    }

    fn set(&mut self, bit_id: u8, o: Arg8) {
    }

    fn res(&mut self, bit_id: u8, o: Arg8) {
    }

    // control
    fn jp(&mut self, dest: u16, cond: Cond) {
    }

    fn jp_hl(&mut self) {
    }

    fn jr(&mut self, offset: u8, cond: Cond) {
    }

    fn call(&mut self, dest: u16, cond: Cond) {
    }

    fn rst(&mut self, offset: u8) {
    }

    fn ret(&mut self, cond: Cond) {
    }

    fn reti(&mut self) {
    }
}

impl Cpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Cpu {
        Cpu {
            running: true,
            memory: memory,
            regs: Registers::new()
        }
    }

    pub fn tick(&mut self) {
        let instr = self.fetch_instr();

        println!("{:?}", instr);

        // TODO: implement execution

        // Stop execution for the lols
        if self.regs.pc > 256 {
            self.running = false;
            self.dump_state();
        }
    }

    // Memory reading helper functions
    fn mem_read_u8(&self, addr: u16) -> u8 {
        self.memory.borrow().read_u8(addr)
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let l = self.mem_read_u8(addr);
        let h = self.mem_read_u8(addr + 1);
        ((l as u16) << 8) | (h as u16)
    }

    fn mem_write_u8(&mut self, addr: u16, data: u8) {
        self.memory.borrow_mut().write_u8(addr, data);
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        self.memory.borrow_mut().write_u16(addr, data);
    }

    pub fn dump_state(&self) {
        println!("Registers:");
        println!("- PC: {:04x} SP: {:04x} ", self.regs.pc, self.regs.sp);
        println!("- A: {:02x} F: {:02x} B: {:02x} C: {:02x}", self.regs.a, self.regs.f, self.regs.b, self.regs.c);
        println!("- D: {:02x} E: {:02x} H: {:02x} L: {:02x}", self.regs.d, self.regs.e, self.regs.h, self.regs.l);
        println!("Flags:");
        println!("- Zero: {}", self.regs.get_flag(Flag::Z));
        println!("- Add/Sub: {}", self.regs.get_flag(Flag::N));
        println!("- Half Carry: {}", self.regs.get_flag(Flag::H));
        println!("- Carry Flag {}", self.regs.get_flag(Flag::C));
    }
}

// Test cases
#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use memory::Memory;
    use super::*;
    use cpu::registers::*;
    use cpu::ops::*;

    fn test_u8() -> u8 {
        144u8
    }

    fn test_u16() -> u16 {
        47628u16
    }

    fn init_cpu() -> Cpu {
        Cpu::new(Rc::new(RefCell::new(Memory::new_blank())))
    }

    #[test]
    fn load_from_reg_a_to_b() {
        let mut cpu = &mut init_cpu();
        cpu.load(Imm8(test_u8()), Reg8::A);
        cpu.load(Reg8::A, Reg8::B);
        assert_eq!(cpu.regs.a, test_u8());
        assert_eq!(cpu.regs.a, cpu.regs.b);
    }

    #[test]
    fn load_from_reg_bc_to_de() {
        let mut cpu = &mut init_cpu();
        cpu.load16(Imm16(test_u16()), Reg16::BC);
        cpu.load16(Reg16::BC, Reg16::DE);
        assert_eq!(Reg16::BC.read(cpu), test_u16());
        assert_eq!(Reg16::BC.read(cpu), Reg16::DE.read(cpu));
    }
}