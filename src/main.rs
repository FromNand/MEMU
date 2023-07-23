mod bus;
mod cartridge;
mod cpu;

use cartridge::Cartridge;
use cpu::CPU;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

fn user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                std::process::exit(0);
            }
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                cpu.write8(0x00ff, b'w');
            }
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                cpu.write8(0x00ff, b's');
            }
            Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                cpu.write8(0x00ff, b'a');
            }
            Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                cpu.write8(0x00ff, b'd');
            }
            _ => {}
        }
    }
}

fn color(index: u8) -> Color {
    match index {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GRAY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

fn screen_changed(cpu: &CPU, frame: &mut [u8; 32 * 32 * 3]) -> bool {
    let mut frame_index = 0;
    let mut update = false;
    for i in 0x0200..0x0600 {
        let color_index = cpu.read8(i as u16);
        let (c1, c2, c3) = color(color_index).rgb();
        if frame[frame_index] != c1 || frame[frame_index + 1] != c2 || frame[frame_index + 2] != c3
        {
            frame[frame_index] = c1;
            frame[frame_index + 1] = c2;
            frame[frame_index + 2] = c3;
            update = true;
        }
        frame_index += 3;
    }
    update
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let window = sdl_context.video().unwrap().window("Snake Game", 640, 640).position_centered().build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(20.0, 20.0).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

    let mut screen = [0 as u8; 32 * 32 * 3];
    let mut random = rand::thread_rng();
    let mut cpu = CPU::new(Cartridge::new(&std::fs::read("cartridge/snake.nes").unwrap()));
    cpu.run(move |cpu| {
        user_input(cpu, &mut event_pump);
        cpu.bus.write8(0x00fe, random.gen_range(1..16));
        if screen_changed(cpu, &mut screen) {
            texture.update(None, &screen, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    });
}
