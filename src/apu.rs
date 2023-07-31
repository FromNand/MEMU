use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

const CPU_HZ: f32 = 1789773.0;

pub struct APU {
    ch1_register: Ch1Register,
    ch1_device: AudioDevice<SquareWave>,
    ch1_sender: Sender<SquareNote>,
}

impl APU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let (ch1_device, ch1_sender) = init_square(&sdl_context);
        APU {
            ch1_register: Ch1Register::new(),
            ch1_device: ch1_device,
            ch1_sender: ch1_sender,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000..=0x4003 => {
                self.ch1_register.write(addr, value);
                self.ch1_sender.send(SquareNote { hz: self.ch1_register.hz(), volume: self.ch1_register.volume(), duty: self.ch1_register.duty() }).unwrap();
            }
            _ => panic!("can't be here"),
        }
    }
}

struct Ch1Register {
    tone_volume: u8,
    sweep: u8,
    hz_low: u8,
    hz_high_key_on: u8,
}

impl Ch1Register {
    fn new() -> Self {
        Ch1Register { tone_volume: 0, sweep: 0, hz_low: 0, hz_high_key_on: 0 }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => self.tone_volume = value,
            0x4001 => self.sweep = value,
            0x4002 => self.hz_low = value,
            0x4003 => self.hz_high_key_on = value,
            _ => panic!("can't be here"),
        }
    }

    fn duty(&self) -> f32 {
        match (self.tone_volume & 0xc0) >> 6 {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => panic!("can't be here"),
        }
    }

    fn volume(&self) -> f32 {
        ((self.tone_volume & 0x0f) as f32) / 15.0
    }

    fn hz(&self) -> f32 {
        let hz = (self.hz_low as u16) + (((self.hz_high_key_on as u16) & 0x07) << 8);
        CPU_HZ / (16.0 * (hz + 1) as f32)
    }
}

struct SquareNote {
    hz: f32,
    volume: f32,
    duty: f32,
}

struct SquareWave {
    phase: f32,
    freq: f32,
    note: SquareNote,
    receiver: Receiver<SquareNote>,
}

impl AudioCallback for SquareWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            match self.receiver.recv_timeout(Duration::from_millis(0)) {
                Ok(note) => self.note = note,
                Err(_) => {},
            }
            *x = if self.phase <= self.note.duty {
                self.note.volume
            } else {
                -self.note.volume
            };
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}


fn init_square(sdl_context: &sdl2::Sdl) -> (AudioDevice<SquareWave>, Sender<SquareNote>) {
    let audio_subsystem = sdl_context.audio().unwrap();
    let (sender, receiver) = channel::<SquareNote>();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave {
            phase: 0.0,
            freq: spec.freq as f32,
            note: SquareNote { hz: 0.0, volume: 0.0, duty: 0.0 },
            receiver: receiver,
        }
    }).unwrap();
    device.resume();
    (device, sender)
}
