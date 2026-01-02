use crate::peripherals::Peripherals;
use crate::cpu::Cpu;

impl Cpu {
    pub fn decode(&mut self, bus: &mut Peripherals) {
        match self.ctx.opcode {
            0x00 => self.nop(bus),
            _ => panic!("Not impremented: {:02X}", self.ctx.opcode),
        }
    }
}