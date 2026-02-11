use crate::{
    Bus,
    decoder::decode,
    instruction::Instruction,
    memory::{ROM_START, SCREEN_HEIGHT, SCREEN_WIDTH},
};

pub struct Cpu {
    v_registers: [u8; 16],
    pc: u16,
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            v_registers: [0; 16],
            pc: ROM_START,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
        }
    }

    fn clear_screen(&mut self, bus: &mut Bus) {
        bus.clear_display();
    }

    fn draw_sprite(&mut self, x: u8, y: u8, n: u8, bus: &mut Bus) {
        let x_coord = self.v_registers[x as usize] % SCREEN_WIDTH as u8;
        let y_coord = self.v_registers[y as usize] % SCREEN_HEIGHT as u8;

        self.v_registers[0xF] = 0;

        for row in 0..n {
            let current_y = y_coord + row;
            if current_y >= SCREEN_HEIGHT as u8 {
                break;
            }

            let sprite_row = bus.memory[self.i as usize + row as usize];

            for bit_idx in 0..8 {
                let current_x = x_coord + bit_idx;
                if current_x >= SCREEN_WIDTH as u8 {
                    break;
                }

                let bit = (sprite_row >> (7 - bit_idx)) & 1;

                if bit == 1 && bus.write_pixel(current_x, current_y, 1) {
                    self.v_registers[0xF] = 1;
                }
            }
        }
    }

    pub fn fetch(&mut self, bus: &Bus) -> u16 {
        let byte1: u16 = (bus.memory[self.pc as usize] as u16) << 8;
        let byte2: u16 = bus.memory[self.pc as usize + 1] as u16;

        // BUG what if it goes past RAM_SIZE?
        // TODO: Move this out of here to somewhere else, maybe main loop or maybe not? Idk bro.
        self.pc += 2;

        byte1 | byte2
    }

    pub fn execute(&mut self, opcode: u16, bus: &mut Bus) -> bool {
        let mut should_redraw = false;
        let instruction =
            decode(opcode).unwrap_or_else(|| panic!("Invalid opcode: {:#06X}", opcode));

        match instruction {
            Instruction::Cls => {
                self.clear_screen(bus);
                should_redraw = true;
            }
            Instruction::Ret => {
                if self.sp == 0 {
                    panic!("The SP cannot be negative!");
                }
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            Instruction::Jump(nnn) => self.pc = nnn,
            Instruction::Call(nnn) => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
            }
            Instruction::SkipEq(x, kk) => {
                if self.v_registers[x as usize] == kk {
                    self.pc += 2;
                }
            }
            Instruction::SkipNotEq(x, kk) => {
                if self.v_registers[x as usize] != kk {
                    self.pc += 2;
                }
            }
            Instruction::SkipRegEq(x, y) => {
                if self.v_registers[x as usize] == self.v_registers[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::Load(x, kk) => self.v_registers[x as usize] = kk,
            Instruction::Add(x, kk) => {
                self.v_registers[x as usize] = self.v_registers[x as usize].wrapping_add(kk)
            }
            Instruction::LoadReg(x, y) => {
                self.v_registers[x as usize] = self.v_registers[y as usize]
            }
            Instruction::Or(x, y) => self.v_registers[x as usize] |= self.v_registers[y as usize],

            Instruction::And(x, y) => self.v_registers[x as usize] &= self.v_registers[y as usize],
            Instruction::Xor(x, y) => self.v_registers[x as usize] ^= self.v_registers[y as usize],
            Instruction::AddReg(x, y) => {
                let result: u16 =
                    self.v_registers[x as usize] as u16 + self.v_registers[y as usize] as u16;
                let carry = if result > 0xFF { 1 } else { 0 };
                self.v_registers[x as usize] = result as u8;
                self.v_registers[0xF] = carry;
            }
            Instruction::SubReg(x, y) => {
                let result =
                    self.v_registers[x as usize].wrapping_sub(self.v_registers[y as usize]);
                let carry = if self.v_registers[x as usize] > self.v_registers[y as usize] {
                    1
                } else {
                    0
                };
                self.v_registers[x as usize] = result;
                self.v_registers[0xF] = carry;
            }
            Instruction::LoadI(nnn) => self.i = nnn,
            Instruction::Draw(x, y, n) => {
                self.draw_sprite(x, y, n, bus);
                should_redraw = true;
            }
            _ => (),
        }

        should_redraw
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Cpu, Bus) {
        (Cpu::new(), Bus::new())
    }

    fn setup_with_sprite(bus: &mut Bus, cpu: &mut Cpu, address: u16, data: u8) {
        bus.memory[address as usize] = data;
        cpu.i = address;
    }

    #[test]
    fn test_fetch_opcode_logic() {
        let (mut cpu, mut bus) = setup();

        let dummy_rom = [0x12, 0x34];

        bus.load_rom(&dummy_rom).unwrap();

        let opcode = cpu.fetch(&bus);
        // bytes should should be successfully fetched and combined into a u16 opcode (Big Endian)
        assert_eq!(opcode, 0x1234);
        // pc should move forward upon reading bytes from memory (2 bytes at a time)
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_op_cls() {
        let (mut cpu, mut bus) = setup();
        setup_with_sprite(&mut bus, &mut cpu, 0x400, 0xF0);

        cpu.v_registers[0] = 10;
        cpu.v_registers[1] = 10;

        // Draw something first
        cpu.execute(0xD011, &mut bus);
        assert_eq!(bus.get_pixel(10, 10), 1);

        // Test the Clear
        cpu.execute(0x00E0, &mut bus);
        assert_eq!(bus.get_pixel(10, 10), 0);
    }

    #[test]
    fn test_op_ret() {
        let (mut cpu, mut bus) = setup();
        cpu.pc = 0x532;
        cpu.sp = 3;
        cpu.stack[cpu.sp as usize] = 0x123;

        let old_sp = cpu.sp;

        cpu.execute(0x00EE, &mut bus);
        assert_eq!(cpu.pc, 0x123);
        assert_eq!(cpu.sp, old_sp - 1);
    }

    #[test]
    fn test_op_1nnn_jump() {
        let (mut cpu, mut bus) = setup();

        cpu.execute(0x1234, &mut bus);
        assert_eq!(cpu.pc, 0x234);
    }

    #[test]
    fn test_op_2nnn_call() {
        let (mut cpu, mut bus) = setup();

        let old_pc = cpu.pc;

        cpu.execute(0x2432, &mut bus);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[cpu.sp as usize], old_pc);
        assert_eq!(cpu.pc, 0x432);
    }

    #[test]
    fn test_op_3xkk_skip_eq() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x6] = 0x78;
        let old_pc = cpu.pc;

        cpu.execute(0x3612, &mut bus);
        assert_ne!(cpu.v_registers[0x6], 0x12);
        assert_eq!(cpu.pc, old_pc);

        cpu.execute(0x3678, &mut bus);
        assert_eq!(cpu.v_registers[0x6], 0x78);
        assert_eq!(cpu.pc, old_pc + 2);
    }

    #[test]
    fn test_op_4xkk_skip_not_eq() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x6] = 0x78;
        let old_pc = cpu.pc;

        cpu.execute(0x4678, &mut bus);
        assert_eq!(cpu.v_registers[0x6], 0x78);
        assert_eq!(cpu.pc, old_pc);

        cpu.execute(0x4612, &mut bus);
        assert_ne!(cpu.v_registers[0x6], 0x12);
        assert_eq!(cpu.pc, old_pc + 2);
    }

    #[test]
    fn test_op_5xy0_skip_reg_eq() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x6] = 0x78;
        cpu.v_registers[0x7] = 0x67;
        let old_pc = cpu.pc;

        cpu.execute(0x5670, &mut bus);
        assert_ne!(cpu.v_registers[0x6], cpu.v_registers[0x7]);
        assert_eq!(cpu.pc, old_pc);

        cpu.v_registers[0x6] = 0x67;
        cpu.execute(0x5670, &mut bus);
        assert_eq!(cpu.v_registers[0x6], cpu.v_registers[0x7]);
        assert_eq!(cpu.pc, old_pc + 2);
    }

    #[test]
    fn test_op_6xkk_load() {
        let (mut cpu, mut bus) = setup();

        cpu.execute(0x6350, &mut bus);
        assert_eq!(cpu.v_registers[3], 0x50);
    }

    #[test]
    fn test_op_7xkk_add() {
        let (mut cpu, mut bus) = setup();
        cpu.v_registers[1] = 0xFE; // 254

        // Add 3 to register 1 (should result in 1, wrapping around)
        cpu.execute(0x7103, &mut bus);

        assert_eq!(cpu.v_registers[1], 0x01);
        assert_eq!(cpu.v_registers[0xF], 0, "VF should NOT be affected by 7XKK");
    }

    #[test]
    fn test_op_8xy0_load() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x7] = 0x42;
        cpu.v_registers[0x4] = 0x00;

        cpu.execute(0x8470, &mut bus);
        assert_eq!(cpu.v_registers[0x4], cpu.v_registers[0x7]);
    }

    #[test]
    fn test_op_8xy1_or() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x7] = 0x42;
        cpu.v_registers[0x4] = 0x54;

        cpu.execute(0x8471, &mut bus);
        assert_eq!(cpu.v_registers[0x4], 0x56);
    }

    #[test]
    fn test_op_8xy2_and() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x7] = 0x42;
        cpu.v_registers[0x4] = 0x54;

        cpu.execute(0x8472, &mut bus);
        assert_eq!(cpu.v_registers[0x4], 0x40);
    }

    #[test]
    fn test_op_8xy3_xor() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x7] = 0x42;
        cpu.v_registers[0x4] = 0x54;

        cpu.execute(0x8473, &mut bus);
        assert_eq!(cpu.v_registers[0x4], 0x16);
    }

    #[test]
    fn test_op_8xy4_add_reg() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x7] = 0x42;
        cpu.v_registers[0x4] = 0x54;

        // carry flag should be 0 at start
        assert_eq!(cpu.v_registers[0xF], 0);
        cpu.execute(0x8474, &mut bus);
        // should still be 0, since result < 255
        assert_eq!(cpu.v_registers[0xF], 0);
        assert_eq!(cpu.v_registers[0x4], 0x96);

        cpu.v_registers[0x7] = 0x96;
        cpu.execute(0x8474, &mut bus);
        // carry flag should be set to 1, since result > 255
        assert_eq!(cpu.v_registers[0xF], 1);
        assert_eq!(cpu.v_registers[0x4], 0x2C);

        cpu.v_registers[0x7] = 0x01;
        cpu.v_registers[0x4] = 0x01;
        cpu.execute(0x8474, &mut bus);
        // carry flag should be set back to 0, since result < 255
        assert_eq!(cpu.v_registers[0xF], 0);
    }

    #[test]
    fn test_op_8xy5_sub_reg() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x4] = 0x54;
        cpu.v_registers[0x7] = 0x42;

        // carry flag should be 0 at start
        assert_eq!(cpu.v_registers[0xF], 0);
        cpu.execute(0x8475, &mut bus);
        // should become 1, since VX > VY
        assert_eq!(cpu.v_registers[0xF], 1);
        assert_eq!(cpu.v_registers[0x4], 0x12);

        cpu.execute(0x8475, &mut bus);
        // carry flag should become 0, since VX < VY
        assert_eq!(cpu.v_registers[0xF], 0);
        // underflow should be handled correctly
        assert_eq!(cpu.v_registers[0x4], 0xD0);
    }

    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    // TODO also look into RESEARCH.md, there are 2 versions of this, the legacy op copies VY into
    // VX and then shifts, the modern op just shifts. So maybe add a config?
    #[test]
    fn test_op_8xy6_shr() {
        let (mut cpu, mut bus) = setup();

        cpu.v_registers[0x4] = 0xA9;
        // carry flag should be 0 at start
        assert_eq!(cpu.v_registers[0xF], 0);
        cpu.execute(0x8476, &mut bus);
        // carry flag should be set to 1, since lsb is 1
        assert_eq!(cpu.v_registers[0xF], 1);
        assert_eq!(cpu.v_registers[0x4], 0x54);

        cpu.execute(0x8476, &mut bus);
        // carry flag should be set to 0, since lsb is 0
        assert_eq!(cpu.v_registers[0xF], 0);
        assert_eq!(cpu.v_registers[0x4], 0x2A);
    }

    #[test]
    fn test_op_annn_load_i() {
        let (mut cpu, mut bus) = setup();

        cpu.execute(0xA123, &mut bus);
        assert_eq!(cpu.i, 0x123);
    }

    #[test]
    fn test_op_dxyn_draw() {
        let (mut cpu, mut bus) = setup();
        setup_with_sprite(&mut bus, &mut cpu, 0x400, 0xF0);

        cpu.v_registers[0] = 10;
        cpu.v_registers[1] = 10;

        cpu.execute(0xD011, &mut bus);
        assert_eq!(bus.get_pixel(10, 10), 1);
        assert_eq!(cpu.v_registers[0xF], 0);

        // Test Collision
        cpu.execute(0xD011, &mut bus);
        assert_eq!(bus.get_pixel(10, 10), 0);
        assert_eq!(cpu.v_registers[0xF], 1);
    }
}
