mod bus;
mod cartridge;
mod cpu;
mod ppu;
mod apu;
mod render;
mod joypad;

use bus::Bus;
use cartridge::Cartridge;
use cpu::CPU;
use render::Frame;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

const A: u8      = 0x01;
const B: u8      = 0x02;
const SELECT: u8 = 0x04;
const START: u8  = 0x08;
const UP: u8     = 0x10;
const DOWN: u8   = 0x20;
const LEFT: u8   = 0x40;
const RIGHT: u8  = 0x80;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let window = sdl_context.video().unwrap().window("MEMU", 256 * 3, 240 * 3).position_centered().build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 256, 240).unwrap();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, DOWN);
    key_map.insert(Keycode::Up, UP);
    key_map.insert(Keycode::Right, RIGHT);
    key_map.insert(Keycode::Left, LEFT);
    key_map.insert(Keycode::Space, SELECT);
    key_map.insert(Keycode::Return, START);
    key_map.insert(Keycode::A, A);
    key_map.insert(Keycode::S, B);

    let target_fps = 60;
    let frame_duration = Duration::from_secs(1) / target_fps;
    let mut last_frame_time = Instant::now();
    let cart = Cartridge::new(&std::fs::read("cartridge/runner.nes").unwrap());
    let mut frame = Frame::new();
    let bus = Bus::new(&sdl_context, cart, move |ppu, joypad| {
        frame.render(ppu);
        texture.update(None, &frame.data, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.status |= *key;
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.status &= !*key;
                    }
                }
                _ => {},
            }
        }
        let elapsed_time = last_frame_time.elapsed();
        if elapsed_time < frame_duration {
            sleep(frame_duration - elapsed_time);
        }
        last_frame_time = Instant::now();
    });
    let mut cpu = CPU::new(bus);
    cpu.run(|cpu| {
        //cpu.log();
    });
}
