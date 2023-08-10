use super::bus::Bus;
use super::cartridge::Cartridge;
use super::cpu::CPU;
use super::render::Frame;
use super::mapper::{Mapper, Mapper0, Mapper1, Mapper2};
use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use lazy_static::lazy_static;

const A: u8      = 0x01;
const B: u8      = 0x02;
const SELECT: u8 = 0x04;
const START: u8  = 0x08;
const UP: u8     = 0x10;
const DOWN: u8   = 0x20;
const LEFT: u8   = 0x40;
const RIGHT: u8  = 0x80;

lazy_static! {
    pub static ref MAPPER: Mutex<Box<Mapper0>> = Mutex::new(Box::new(Mapper0::new()));
    //pub static ref MAPPER: Mutex<Box<Mapper1>> = Mutex::new(Box::new(Mapper1::new()));
    //pub static ref MAPPER: Mutex<Box<Mapper2>> = Mutex::new(Box::new(Mapper2::new()));
}

pub fn nes() {
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
    let interval = 1000 * 1000 * 1000 / target_fps;
    let mut now = Instant::now();
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/alter.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/bomb.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/Dodgeball.nes").unwrap()); //X
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/DQ2.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/DQ3.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/DQ4.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/FE.nes").unwrap()); //X
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/FF3.nes").unwrap()); //X
    let cart = Cartridge::new(&std::fs::read("cartridge/nes/mario.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/Mario3.nes").unwrap()); //X
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/MarioUSA.nes").unwrap()); //X
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/Mother.nes").unwrap()); //X
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/runner.nes").unwrap());
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/TwinBee.nes").unwrap()); //X
    // let cart = Cartridge::new(&std::fs::read("cartridge/nes/Zelda.nes").unwrap()); //X
    MAPPER.lock().unwrap().cart = cart;
    let mut frame = Frame::new();
    let bus = Bus::new(&sdl_context, move |ppu, joypad| {
        frame.render(ppu);
        texture.update(None, &frame.data, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    MAPPER.lock().unwrap().backup();
                    std::process::exit(0);
                }
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
        let time = now.elapsed().as_nanos();
        if time < interval {
            sleep(Duration::from_nanos((interval - time) as u64));
        }
        now = Instant::now();
    });
    let mut cpu = CPU::new(bus);
    cpu.run(|cpu| {
        //cpu.log();
    });
    MAPPER.lock().unwrap().backup();
}
