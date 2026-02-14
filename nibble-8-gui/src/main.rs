extern crate sdl2;

use nibble_8_core::cpu::ThreadRngSource;
use nibble_8_core::memory::{SCREEN_HEIGHT, SCREEN_WIDTH};
use nibble_8_core::{Bus, Cpu};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::fs::read;
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Nibble-8", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut cpu = Cpu::new(Box::new(ThreadRngSource::new()));
    let mut bus = Bus::new();
    let rom_vec = read("./roms/mySnake.ch8").expect("Failed to read ROM file");
    bus.load_rom(&rom_vec).unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    scancode: Some(k),
                    repeat: false,
                    ..
                } => {
                    if let Some(chip8_key) = map_keycode_to_chip8(k) {
                        bus.set_key(chip8_key, true);
                    }
                }

                Event::KeyUp {
                    scancode: Some(k), ..
                } => {
                    if let Some(chip8_key) = map_keycode_to_chip8(k) {
                        bus.set_key(chip8_key, false);
                    }
                }

                _ => {}
            }
        }

        let mut frame_needs_redraw = false;

        for _ in 0..10 {
            let opcode = cpu.fetch(&bus);
            if cpu.execute(opcode, &mut bus) {
                frame_needs_redraw = true;
            }
        }

        cpu.decrease_timers();

        if frame_needs_redraw {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));

            for y in 0..SCREEN_HEIGHT {
                for x in 0..SCREEN_WIDTH {
                    if bus.get_pixel(x, y) == 1 {
                        let rect = Rect::new((x * 10) as i32, (y * 10) as i32, 10, 10);
                        canvas.fill_rect(rect).unwrap();
                    }
                }
            }
            canvas.present();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn map_keycode_to_chip8(k: Scancode) -> Option<u8> {
    Some(match k {
        Scancode::Num1 => 0x1,
        Scancode::Num2 => 0x2,
        Scancode::Num3 => 0x3,
        Scancode::Num4 => 0xC,

        Scancode::Q => 0x4,
        Scancode::W => 0x5,
        Scancode::E => 0x6,
        Scancode::R => 0xD,

        Scancode::A => 0x7,
        Scancode::S => 0x8,
        Scancode::D => 0x9,
        Scancode::F => 0xE,

        Scancode::Z => 0xA,
        Scancode::X => 0x0,
        Scancode::C => 0xB,
        Scancode::V => 0xF,

        _ => return None,
    })
}
