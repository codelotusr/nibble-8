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
}
