use crate::{cpu::register::Registers, peripherals::Peripherals};

mod fetch;
mod register;
mod decode;
mod instructions;
mod operand;

#[derive(Default)]
struct Ctx {
    opcode: u8,
    cb: bool,
}
pub struct Cpu {
    regs: Registers,
    ctx: Ctx,
}

impl Cpu {
    pub fn emulate_cycle(&mut self, bus: &mut Peripherals) {
        self.decode(bus);
    }
}