use crate::instruction::Instruction;

#[derive(Debug, PartialEq)]
struct OpcodeComponents {
    op: u8,
    x: u8,
    y: u8,
    n: u8,
    kk: u8,
    nnn: u16,
}

impl From<u16> for OpcodeComponents {
    fn from(opcode: u16) -> Self {
        OpcodeComponents {
            op: ((opcode & 0xF000) >> 12) as u8,
            x: ((opcode & 0x0F00) >> 8) as u8,
            y: ((opcode & 0x00F0) >> 4) as u8,
            n: (opcode & 0x000F) as u8,
            kk: (opcode & 0x00FF) as u8,
            nnn: opcode & 0x0FFF,
        }
    }
}

pub fn decode(opcode: u16) -> Option<Instruction> {
    let opcode_components = OpcodeComponents::from(opcode);
    match opcode_components.op {
        0x0 => match opcode_components.kk {
            0x00E0 => Some(Instruction::Cls),
            0x00EE => Some(Instruction::Ret),
            _ => None,
        },
        0x1 => Some(Instruction::Jump(opcode_components.nnn)),
        0x2 => Some(Instruction::Call(opcode_components.nnn)),
        0x3 => Some(Instruction::SkipEq(
            opcode_components.x,
            opcode_components.kk,
        )),
        0x4 => Some(Instruction::SkipNotEq(
            opcode_components.x,
            opcode_components.kk,
        )),
        0x5 => match opcode_components.n {
            0x0 => Some(Instruction::SkipRegEq(
                opcode_components.x,
                opcode_components.y,
            )),
            _ => None,
        },
        0x6 => Some(Instruction::Load(opcode_components.x, opcode_components.kk)),
        0x7 => Some(Instruction::Add(opcode_components.x, opcode_components.kk)),
        0x8 => match opcode_components.n {
            0x0 => Some(Instruction::LoadReg(
                opcode_components.x,
                opcode_components.y,
            )),
            0x1 => Some(Instruction::Or(opcode_components.x, opcode_components.y)),
            0x2 => Some(Instruction::And(opcode_components.x, opcode_components.y)),
            0x3 => Some(Instruction::Xor(opcode_components.x, opcode_components.y)),
            0x4 => Some(Instruction::AddReg(
                opcode_components.x,
                opcode_components.y,
            )),
            0x5 => Some(Instruction::SubReg(
                opcode_components.x,
                opcode_components.y,
            )),
            0x6 => Some(Instruction::Shr(opcode_components.x, opcode_components.y)),
            0x7 => Some(Instruction::Subn(opcode_components.x, opcode_components.y)),
            0xE => Some(Instruction::Shl(opcode_components.x, opcode_components.y)),
            _ => None,
        },
        0x9 => match opcode_components.n {
            0x0 => Some(Instruction::SkipRegNotEq(
                opcode_components.x,
                opcode_components.y,
            )),
            _ => None,
        },
        0xA => Some(Instruction::LoadI(opcode_components.nnn)),
        0xB => Some(Instruction::JumpOffset(opcode_components.nnn)),
        0xC => Some(Instruction::Rand(opcode_components.x, opcode_components.kk)),
        0xD => Some(Instruction::Draw(
            opcode_components.x,
            opcode_components.y,
            opcode_components.n,
        )),
        0xE => match opcode_components.kk {
            0x9E => Some(Instruction::SkipIfPressed(opcode_components.x)),
            0xA1 => Some(Instruction::SkipIfNotPressed(opcode_components.x)),
            _ => None,
        },
        0xF => match opcode_components.kk {
            0x07 => Some(Instruction::LoadRegFromDelay(opcode_components.x)),
            0x0A => Some(Instruction::WaitForKey(opcode_components.x)),
            0x15 => Some(Instruction::LoadDelayFromReg(opcode_components.x)),
            0x18 => Some(Instruction::LoadSoundFromReg(opcode_components.x)),
            0x1E => Some(Instruction::AddIndex(opcode_components.x)),
            0x29 => Some(Instruction::LoadFont(opcode_components.x)),
            0x33 => Some(Instruction::Bcd(opcode_components.x)),
            0x55 => Some(Instruction::DumpRegs(opcode_components.x)),
            0x65 => Some(Instruction::FillRegs(opcode_components.x)),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_accurate_opcode_component_constructor() {
        let opcode = 0xD123;
        let opcode_components = OpcodeComponents::from(opcode);

        let expected = OpcodeComponents {
            op: 0xD,
            x: 0x1,
            y: 0x2,
            n: 0x3,
            kk: 0x23,
            nnn: 0x123,
        };

        assert_eq!(opcode_components, expected);
    }

    #[test]
    fn test_decode_opcode() {
        let opcode = 0xD123;

        let decoded = decode(opcode);
        assert_eq!(decoded, Some(Instruction::Draw(0x1, 0x2, 0x3)));
    }
}
