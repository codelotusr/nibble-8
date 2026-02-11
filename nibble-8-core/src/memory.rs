pub const RAM_SIZE: u16 = 4096;
pub const FONT_BASE: u16 = 0x050;
pub const ROM_START: u16 = 0x200;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const KEY_COUNT: usize = 16;

// 5x16
pub const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Display {
    display_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

struct Keypad {
    keys: [bool; 16],
}

impl Keypad {
    fn new() -> Self {
        Self {
            keys: [false; KEY_COUNT],
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn set_key(&mut self, key: u8, pressed: bool) {
        self.keys[key as usize] = pressed;
    }
}

pub struct Bus {
    pub memory: [u8; RAM_SIZE as usize],
    display: Display,
    keypad: Keypad,
}

impl Bus {
    pub fn new() -> Self {
        let mut bus = Self {
            memory: [0; RAM_SIZE as usize],
            display: Display {
                display_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            },
            keypad: Keypad::new(),
        };

        for (i, &byte) in FONTSET.iter().enumerate() {
            bus.memory[FONT_BASE as usize + i] = byte;
        }

        bus
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        let available_space = RAM_SIZE - ROM_START;
        if rom.len() > available_space as usize {
            return Err("The ROM is too big".to_string());
        }

        self.memory[ROM_START as usize..ROM_START as usize + rom.len()].copy_from_slice(rom);

        Ok(())
    }

    pub fn write_pixel(&mut self, x: u8, y: u8, value: u8) -> bool {
        let index = (y as usize * SCREEN_WIDTH) + x as usize;
        let old_pixel = self.display.display_buffer[index];

        self.display.display_buffer[index] ^= value;

        old_pixel == 1 && self.display.display_buffer[index] == 0
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let index = (y * SCREEN_WIDTH) + x;
        self.display.display_buffer[index]
    }

    pub fn clear_display(&mut self) {
        self.display.display_buffer.fill(0);
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.keypad.is_pressed(key)
    }

    pub fn set_key(&mut self, key: u8, pressed: bool) {
        self.keypad.set_key(key, pressed);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fontset_is_loaded() {
        let bus = Bus::new();

        // The first byte of the FONTSET (for '0') should be 0xF0
        assert_eq!(bus.memory[FONT_BASE as usize], 0xF0);

        // The last byte (for 'F') should be 0x80
        let last_idx = FONT_BASE as usize + FONTSET.len() - 1;
        assert_eq!(bus.memory[last_idx], 0x80);
    }

    #[test]
    fn test_rom_is_too_big() {
        let mut bus = Bus::new();

        // The ROM size must not exceed available space (3584 bytes)
        assert_eq!(
            bus.load_rom(&[0; 4000]),
            Err("The ROM is too big".to_string())
        );
    }

    #[test]
    fn test_rom_loads_correctly() {
        let mut bus = Bus::new();
        let dummy_rom = [0x11, 0x12, 0x13, 0x14, 0x15];

        bus.load_rom(&dummy_rom).unwrap();
        // The ROM should be loaded into the memory and all the bytes should match
        assert_eq!(
            &bus.memory[ROM_START as usize..ROM_START as usize + dummy_rom.len()],
            dummy_rom
        );
    }

    #[test]
    fn test_pixel_write_correctly() {}
}
