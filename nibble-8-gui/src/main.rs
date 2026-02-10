extern crate sdl2;

use nibble_8_core::memory::{SCREEN_HEIGHT, SCREEN_WIDTH};
use nibble_8_core::{Bus, Cpu};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
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

    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    let rom_vec = read("./roms/2-ibm-logo.ch8").expect("Failed to read ROM file");
    bus.load_rom(&rom_vec).unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } = event
            {
                break 'running;
            }
        }

        let mut frame_needs_redraw = false;

        for _ in 0..10 {
            let opcode = cpu.fetch(&bus);
            if cpu.execute(opcode, &mut bus) {
                frame_needs_redraw = true;
            }
        }

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
