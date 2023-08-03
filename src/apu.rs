use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};
use lazy_static::lazy_static;

const CPU_HZ: f32 = 1789772.5;

pub struct APU {
    ch1_register: Ch1Register,
    ch1_device: AudioDevice<SquareWave>,
    ch1_sender: Sender<SquareEvent>,
    ch2_register: Ch2Register,
    ch2_device: AudioDevice<SquareWave>,
    ch2_sender: Sender<SquareEvent>,
    ch3_register: Ch3Register,
    ch3_device: AudioDevice<TriangleWave>,
    ch3_sender: Sender<TriangleEvent>,
    ch4_register: Ch4Register,
    ch4_device: AudioDevice<NoiseWave>,
    ch4_sender: Sender<NoiseEvent>,
    frame_counter: FrameCounter,
    cycles: usize,
    counter: usize,
    status: StatusRegister,
}

impl APU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let (ch1_device, ch1_sender) = init_square(sdl_context);
        let (ch2_device, ch2_sender) = init_square(sdl_context);
        let (ch3_device, ch3_sender) = init_triangle(sdl_context);
        let (ch4_device, ch4_sender) = init_noise(sdl_context);
        APU {
            ch1_register: Ch1Register::new(),
            ch1_device,
            ch1_sender,
            ch2_register: Ch2Register::new(),
            ch2_device,
            ch2_sender,
            ch3_register: Ch3Register::new(),
            ch3_device,
            ch3_sender,
            ch4_register: Ch4Register::new(),
            ch4_device,
            ch4_sender,
            frame_counter: FrameCounter::new(),
            cycles: 0,
            counter: 0,
            status: StatusRegister::new(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000..=0x4003 => {
                self.ch1_register.write(addr, value);
                self.ch1_sender.send(SquareEvent::Note(SquareNote { duty: self.ch1_register.duty })).unwrap();
                self.ch1_sender.send(SquareEvent::Envelope(Envelope::new(self.ch1_register.volume, self.ch1_register.envelope_flag, !self.ch1_register.key_off_counter_flag))).unwrap();
                self.ch1_sender.send(SquareEvent::LengthCounter(LengthCounter::new(self.ch1_register.key_off_counter_flag, LENGTH_COUNTER_TABLE[self.ch1_register.key_off_count as usize]))).unwrap();
                self.ch1_sender.send(SquareEvent::Sweep(Sweep::new(self.ch1_register.frequency, self.ch1_register.sweep_change_amount, self.ch1_register.sweep_direction,
                    self.ch1_register.sweep_timer_count, 0, self.ch1_register.sweep_enabled))).unwrap();
                if addr == 0x4003 {
                    self.ch1_sender.send(SquareEvent::Reset()).unwrap();
                }
            }
            0x4004..=0x4007 => {
                self.ch2_register.write(addr, value);
                self.ch2_sender.send(SquareEvent::Note(SquareNote { duty: self.ch2_register.duty })).unwrap();
                self.ch2_sender.send(SquareEvent::Envelope(Envelope::new(self.ch2_register.volume, self.ch2_register.envelope_flag, !self.ch2_register.key_off_counter_flag))).unwrap();
                self.ch2_sender.send(SquareEvent::LengthCounter(LengthCounter::new(self.ch2_register.key_off_counter_flag, LENGTH_COUNTER_TABLE[self.ch2_register.key_off_count as usize]))).unwrap();
                self.ch2_sender.send(SquareEvent::Sweep(Sweep::new(self.ch2_register.frequency, self.ch2_register.sweep_change_amount, self.ch2_register.sweep_direction,
                    self.ch2_register.sweep_timer_count, 0, self.ch2_register.sweep_enabled))).unwrap();
                if addr == 0x4007 {
                    self.ch2_sender.send(SquareEvent::Reset()).unwrap();
                }
            }
            0x4008 | 0x400a | 0x400b => {
                self.ch3_register.write(addr, value);
                self.ch3_sender.send(TriangleEvent::Note(TriangleNote { hz: self.ch3_register.hz() })).unwrap();
                self.ch3_sender.send(TriangleEvent::LengthCounter(LengthCounter::new(self.ch3_register.key_off_counter_flag, LENGTH_COUNTER_TABLE[self.ch3_register.key_off_count as usize]))).unwrap();
                if addr == 0x400b {
                    self.ch3_sender.send(TriangleEvent::Reset()).unwrap();
                }
            }
            0x400c | 0x400e | 0x400f => {
                self.ch4_register.write(addr, value);
                self.ch4_sender.send(NoiseEvent::Note(NoiseNote { hz: CPU_HZ / NOISE_TABLE[self.ch4_register.hz as usize] as f32, volume: self.ch4_register.volume as f32 / 15.0, is_long: self.ch4_register.noise_type == NoiseType::LONG })).unwrap();
                self.ch4_sender.send(NoiseEvent::Envelope(Envelope::new(self.ch4_register.volume, self.ch4_register.envelope_flag, !self.ch4_register.key_off_counter_flag))).unwrap();
                self.ch4_sender.send(NoiseEvent::LengthCounter(LengthCounter::new(self.ch4_register.key_off_counter_flag, LENGTH_COUNTER_TABLE[self.ch4_register.key_off_count as usize]))).unwrap();
                if addr == 0x400f {
                    self.ch4_sender.send(NoiseEvent::Reset()).unwrap();
                }
            }
            _ => panic!("can't be here"),
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let return_value = self.status.get();
        self.status.enable_frame_irq = false;
        return_value
    }

    pub fn write_status(&mut self, data: u8) {
        self.status.update(data);
        self.ch1_sender.send(SquareEvent::Enable(self.status.enable_ch1)).unwrap();
        self.ch2_sender.send(SquareEvent::Enable(self.status.enable_ch2)).unwrap();
        self.ch3_sender.send(TriangleEvent::Enable(self.status.enable_ch3)).unwrap();
        self.ch4_sender.send(NoiseEvent::Enable(self.status.enable_ch4)).unwrap();
    }

    pub fn irq(&self) -> bool {
        self.status.enable_frame_irq
    }

    pub fn write_frame_counter(&mut self, data: u8) {
        self.frame_counter.update(data);
        self.cycles = 0;
        self.counter = 0;
    }

    fn send_length_counter_tick(&self) {
        self.ch1_sender.send(SquareEvent::LengthCounterTick()).unwrap();
        self.ch2_sender.send(SquareEvent::LengthCounterTick()).unwrap();
        self.ch3_sender.send(TriangleEvent::LengthCounterTick()).unwrap();
        self.ch4_sender.send(NoiseEvent::LengthCounterTick()).unwrap();
    }

    fn send_sweep_tick(&self) {
        self.ch1_sender.send(SquareEvent::SweepTick()).unwrap();
        self.ch2_sender.send(SquareEvent::SweepTick()).unwrap();
    }

    fn send_envelope_tick(&self) {
        self.ch1_sender.send(SquareEvent::EnvelopeTick()).unwrap();
        self.ch2_sender.send(SquareEvent::EnvelopeTick()).unwrap();
        self.ch4_sender.send(NoiseEvent::EnvelopeTick()).unwrap();
    }

    pub fn tick(&mut self, cycle: usize) {
        self.cycles = self.cycles.wrapping_add(cycle);
        let interval = 7457;
        if self.cycles >= interval {
            self.cycles -= interval;
            self.counter += 1;
            match self.frame_counter.mode() {
                4 => {
                    if self.counter == 2 || self.counter == 4 {
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter == 4 {
                        self.counter = 0;
                        self.status.enable_frame_irq = true;
                    }
                    self.send_envelope_tick();
                }
                5 => {
                    if self.counter == 1 || self.counter == 3 {
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter <= 4 {
                        self.send_envelope_tick();
                    }
                    if self.counter == 5 {
                        self.counter = 0;
                    }
                }
                _ => panic!("can't be here"),
            }
        }
    }
}







struct Ch1Register {
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,
    sweep_change_amount: u8,
    sweep_direction: bool,
    sweep_timer_count: u8,
    sweep_enabled: bool,
    frequency: u16,
    key_off_count: u8,
}

impl Ch1Register {
    fn new() -> Self {
        Ch1Register { volume: 0, envelope_flag: false, key_off_counter_flag: true, duty: 0, sweep_change_amount: 0, sweep_direction: false, sweep_timer_count: 0, sweep_enabled: false, frequency: 0, key_off_count: 0 }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => {
                self.volume = value & 0x0f;
                self.envelope_flag = value & 0x10 == 0;
                self.key_off_counter_flag = value & 0x20 == 0;
                self.duty = (value & 0xc0) >> 6;
            }
            0x4001 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_direction = value & 0x08 != 0;
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enabled = (value & 0x80) != 0;
            }
            0x4002 => self.frequency = (self.frequency & 0xff00) + value as u16,
            0x4003 => {
                self.frequency = (self.frequency & 0x00ff) + ((value as u16 & 0x07) << 8);
                self.key_off_count = (value & 0xf8) >> 3;
            }
            _ => panic!("can't be here"),
        }
    }
}

struct Envelope {
    rate: u8,
    enabled: bool,
    loop_flag: bool,
    counter: u8,
    division_period: u8,
}

impl Envelope {
    fn new(rate: u8, enabled: bool, loop_flag: bool) -> Self {
        Envelope { rate, enabled, loop_flag, counter: 0x0f, division_period: rate + 1, }
    }

    fn tick(&mut self) {
        self.division_period -= 1;
        if self.division_period != 0 {
            return;
        }
        if self.counter != 0 {
            self.counter -= 0;
        } else if self.loop_flag {
            self.reset();
        }
        self.division_period = self.rate + 1;
    }

    fn volume(&self) -> f32 {
        (if self.enabled {
            self.counter
        } else {
            self.rate
        }) as f32 / 15.0
    }

    fn reset(&mut self) {
        self.counter = 0x0f;
        self.division_period = self.rate + 1;
    }
}

lazy_static! {
    pub static ref LENGTH_COUNTER_TABLE: Vec<u8> = vec![
        0x05, 0x7f, 0x0a, 0x01, 0x14, 0x02, 0x28, 0x03, 0x50, 0x04, 0x1e, 0x05, 0x07, 0x06, 0x0d, 0x07,
        0x06, 0x08, 0x0c, 0x09, 0x18, 0x0a, 0x30, 0x0b, 0x60, 0x0c, 0x24, 0x0d, 0x08, 0x0e, 0x10, 0x0f,
    ];
}

struct LengthCounter {
    enabled: bool,
    count: u8,
    counter: u8,
}

impl LengthCounter {
    fn new(enabled: bool, counter: u8) -> Self {
        LengthCounter { enabled, count: counter, counter }
    }

    fn tick(&mut self) {
        if self.enabled {
            if self.counter > 0 {
                self.counter -= 1;
            }
        }
    }

    fn mute(&self) -> bool {
        self.enabled && self.counter == 0
    }

    fn reset(&mut self) {
        self.counter = self.count;
    }
}

struct Sweep {
    org_freq: u16,
    frequency: u16,
    change_amount: u8,
    direction: bool,
    timer_count: u8,
    counter: u8,
    enabled: bool,
}

impl Sweep {
    fn new(frequency: u16, change_amount: u8, direction: bool, timer_count: u8, counter: u8, enabled: bool) -> Self {
        Sweep { org_freq: frequency, frequency, change_amount, direction, timer_count, counter, enabled }
    }

    fn tick(&mut self) {
        if self.enabled == false {
            return;
        }
        if self.change_amount == 0 {
            return;
        }
        self.counter += 1;
        if self.counter <= self.timer_count + 1 {
            return;
        }
        self.counter = 0;
        if self.direction == false {
            self.frequency = self.frequency + (self.frequency >> self.change_amount);
        } else {
            self.frequency = self.frequency - (self.frequency >> self.change_amount);
        }
        if self.frequency < 8 || self.frequency > 0x7ff {
            self.frequency = 0;
        }
    }

    fn hz(&self) -> f32 {
        if self.frequency == 0 {
            return 0.0
        }
        CPU_HZ / (16.0 * (self.frequency + 1) as f32)
    }

    fn reset(&mut self) {
        self.frequency = self.org_freq;
        self.counter = 0;
    }
}

enum SquareEvent {
    Note(SquareNote),
    Enable(bool),
    Envelope(Envelope),
    EnvelopeTick(),
    LengthCounter(LengthCounter),
    LengthCounterTick(),
    Sweep(Sweep),
    SweepTick(),
    Reset(),
}

struct SquareNote {
    duty: u8,
}

impl SquareNote {
    fn new() -> Self {
        SquareNote { duty: 0 }
    }

    fn duty(&self) -> f32 {
        match self.duty {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => panic!("can't be here"),
        }
    }
}

struct SquareWave {
    phase: f32,
    freq: f32,
    note: SquareNote,
    envelope: Envelope,
    enabled: bool,
    length_counter: LengthCounter,
    sweep: Sweep,
    receiver: Receiver<SquareEvent>,
}

impl AudioCallback for SquareWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            loop {
                match self.receiver.recv_timeout(Duration::from_millis(0)) {
                    Ok(SquareEvent::Note(note)) => self.note = note,
                    Ok(SquareEvent::Enable(b)) => self.enabled = b,
                    Ok(SquareEvent::Envelope(e)) => self.envelope = e,
                    Ok(SquareEvent::EnvelopeTick()) => self.envelope.tick(),
                    Ok(SquareEvent::LengthCounter(l)) => self.length_counter = l,
                    Ok(SquareEvent::LengthCounterTick()) => self.length_counter.tick(),
                    Ok(SquareEvent::Sweep(s)) => self.sweep = s,
                    Ok(SquareEvent::SweepTick()) => self.sweep.tick(),
                    Ok(SquareEvent::Reset()) => {
                        self.envelope.reset();
                        self.length_counter.reset();
                        self.sweep.reset();
                    }
                    Err(_) => break,
                }
            }
            *x = if self.phase <= self.note.duty() {
                self.envelope.volume()
            } else {
                -self.envelope.volume()
            };
            if self.length_counter.mute() {
                *x = 0.0;
            }
            if self.enabled == false {
                *x = 0.0;
            }
            let hz = self.sweep.hz();
            if hz != 0.0 {
                self.phase = (self.phase + hz / self.freq) % 1.0;
            }
        }
    }
}

fn init_square(sdl_context: &sdl2::Sdl) -> (AudioDevice<SquareWave>, Sender<SquareEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();
    let (sender, receiver) = channel::<SquareEvent>();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave {
            phase: 0.0,
            freq: spec.freq as f32,
            note: SquareNote { duty: 0 },
            envelope: Envelope::new(0, false, false),
            enabled: true,
            length_counter: LengthCounter::new(false, 0),
            sweep: Sweep::new(0, 0, false, 0, 0, false),
            receiver: receiver,
        }
    }).unwrap();
    device.resume();
    (device, sender)
}






struct Ch2Register {
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,
    sweep_change_amount: u8,
    sweep_direction: bool,
    sweep_timer_count: u8,
    sweep_enabled: bool,
    frequency: u16,
    key_off_count: u8,
}

impl Ch2Register {
    fn new() -> Self {
        Ch2Register { volume: 0, envelope_flag: false, key_off_counter_flag: true, duty: 0, sweep_change_amount: 0, sweep_direction: false, sweep_timer_count: 0, sweep_enabled: false, frequency: 0, key_off_count: 0 }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4004 => {
                self.volume = value & 0x0f;
                self.envelope_flag = value & 0x10 == 0;
                self.key_off_counter_flag = value & 0x20 == 0;
                self.duty = (value & 0xc0) >> 6;
            }
            0x4005 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_direction = value & 0x08 != 0;
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enabled = (value & 0x80) != 0;
            }
            0x4006 => self.frequency = (self.frequency & 0xff00) + value as u16,
            0x4007 => {
                self.frequency = (self.frequency & 0x00ff) + ((value as u16 & 0x07) << 8);
                self.key_off_count = (value & 0xf8) >> 3;
            }
            _ => panic!("can't be here"),
        }
    }

    fn hz(&self) -> f32 {
        CPU_HZ / (16.0 * (self.frequency + 1) as f32)
    }
}








struct Ch3Register {
    length: u8,
    key_off_counter_flag: bool,
    hz: u16,
    key_off_count: u8,
}

impl Ch3Register {
    fn new() -> Self {
        Ch3Register { length: 0, key_off_counter_flag: true, hz: 0, key_off_count: 0 }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4008 => {
                self.length = value & 0x7f;
                self.key_off_counter_flag = value & 0x80 == 0;
            }
            0x400a => self.hz = (self.hz & 0xff00) + value as u16,
            0x400b => {
                self.hz = (self.hz & 0x00ff) + ((value as u16 & 0x07) << 8);
                self.key_off_count = (value & 0xf8) >> 3;
            }
            _ => panic!("can't be here"),
        }
    }

    fn hz(&self) -> f32 {
        CPU_HZ / (32.0 * (self.hz + 1) as f32)
    }
}

enum TriangleEvent {
    Note(TriangleNote),
    Enable(bool),
    LengthCounter(LengthCounter),
    LengthCounterTick(),
    Reset(),
}

struct TriangleNote {
    hz: f32,
}

struct TriangleWave {
    freq: f32,
    phase: f32,
    enabled: bool,
    length_counter: LengthCounter,
    receiver: Receiver<TriangleEvent>,
    note: TriangleNote,
}

impl AudioCallback for TriangleWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            loop {
                match self.receiver.recv_timeout(Duration::from_millis(0)) {
                    Ok(TriangleEvent::Note(note)) => self.note = note,
                    Ok(TriangleEvent::Enable(b)) => self.enabled = b,
                    Ok(TriangleEvent::LengthCounter(l)) => self.length_counter = l,
                    Ok(TriangleEvent::LengthCounterTick()) => self.length_counter.tick(),
                    Ok(TriangleEvent::Reset()) => {
                        self.length_counter.reset();
                    }
                    Err(_) => break,
                }
            }
            *x = (if self.phase <= 0.5 {
                self.phase
            } else {
                1.0 - self.phase
            } - 0.25) * 4.0;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
            if self.enabled == false {
                *x = 0.0;
            }
            if self.length_counter.mute() {
                *x = 0.0;
            }
        }
    }
}

fn init_triangle(sdl_context: &sdl2::Sdl) -> (AudioDevice<TriangleWave>, Sender<TriangleEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();
    let (sender, receiver) = channel::<TriangleEvent>();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        TriangleWave {
            phase: 0.0,
            freq: spec.freq as f32,
            note: TriangleNote { hz: 0.0 },
            enabled: true,
            length_counter: LengthCounter::new(false, 0),
            receiver: receiver,
        }
    }).unwrap();
    device.resume();
    (device, sender)
}








#[derive(PartialEq)]
enum NoiseType {
    LONG, SHORT
}

lazy_static! {
    pub static ref NOISE_TABLE: Vec<u16> = vec![
        0x002, 0x004, 0x008, 0x010, 0x020, 0x030, 0x040, 0x050, 0x065, 0x07f, 0x0be, 0x0fe, 0x17d, 0x1fc, 0x3f9, 0x7f2
    ];
}

struct Ch4Register {
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    hz: u8,
    noise_type: NoiseType,
    key_off_count: u8
}

impl Ch4Register {
    fn new() -> Self {
        Ch4Register { volume: 0, envelope_flag: false, key_off_counter_flag: true, hz: 0, noise_type: NoiseType::LONG, key_off_count: 0 }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x400c => {
                self.volume = value & 0x0f;
                self.envelope_flag = value & 0x10 == 0;
                self.key_off_counter_flag = value & 0x20 == 0;
            }
            0x400e => {
                self.hz = value & 0x0f;
                self.noise_type = if value & 0x80 != 0 { NoiseType::SHORT } else { NoiseType::LONG };
            }
            0x400f => {
                self.key_off_count = (value & 0xf8) >> 3;
            }
            _ => panic!("can't be here"),
        }
    }
}

enum NoiseEvent {
    Note(NoiseNote),
    Enable(bool),
    Envelope(Envelope),
    EnvelopeTick(),
    LengthCounter(LengthCounter),
    LengthCounterTick(),
    Reset(),
}

struct NoiseNote {
    hz: f32,
    volume: f32,
    is_long: bool,
}

struct NoiseRandom {
    bit: u8,
    value: u16,
}

impl NoiseRandom {
    fn new_long() -> Self {
        NoiseRandom { bit: 1, value: 1 }
    }

    fn new_short() -> Self {
        NoiseRandom { bit: 6, value: 1 }
    }

    fn next(&mut self) -> bool {
        self.value = (self.value >> 1) + (((self.value ^ (self.value >> self.bit)) & 0x01) << 14);
        self.value & 0x01 == 0
    }
}

struct NoiseWave {
    value: bool,
    phase: f32,
    freq: f32,
    note: NoiseNote,
    short_random: NoiseRandom,
    long_random: NoiseRandom,
    enabled: bool,
    envelope: Envelope,
    length_counter: LengthCounter,
    receiver: Receiver<NoiseEvent>,
}

impl AudioCallback for NoiseWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            loop {
                match self.receiver.recv_timeout(Duration::from_millis(0)) {
                    Ok(NoiseEvent::Note(note)) => self.note = note,
                    Ok(NoiseEvent::Enable(b)) => self.enabled = b,
                    Ok(NoiseEvent::Envelope(e)) => self.envelope = e,
                    Ok(NoiseEvent::EnvelopeTick()) => self.envelope.tick(),
                    Ok(NoiseEvent::LengthCounter(l)) => self.length_counter = l,
                    Ok(NoiseEvent::LengthCounterTick()) => self.length_counter.tick(),
                    Ok(NoiseEvent::Reset()) => {
                        self.envelope.reset();
                        self.length_counter.reset();
                    }
                    Err(_) => break,
                }
            }
            let old_phase = self.phase;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
            if old_phase > self.phase {
                self.value = if self.note.is_long {
                    self.long_random.next()
                } else {
                    self.short_random.next()
                };
            }
            *x = (self.value as u32 as f32) * self.envelope.volume();
            if self.enabled == false {
                *x = 0.0;
            }
            if self.length_counter.mute() {
                *x = 0.0;
            }
        }
    }
}

fn init_noise(sdl_context: &sdl2::Sdl) -> (AudioDevice<NoiseWave>, Sender<NoiseEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();
    let (sender, receiver) = channel::<NoiseEvent>();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        NoiseWave {
            value: false,
            phase: 0.0,
            freq: spec.freq as f32,
            note: NoiseNote { hz: 0.0, volume: 0.0, is_long: true },
            short_random: NoiseRandom::new_short(),
            long_random: NoiseRandom::new_long(),
            enabled: true,
            envelope: Envelope::new(0, false, false),
            length_counter: LengthCounter::new(false, 0),
            receiver: receiver,
        }
    }).unwrap();
    device.resume();
    (device, sender)
}







struct FrameCounter {
    disable_irq: bool,
    sequence_mode: bool,
}

impl FrameCounter {
    fn new() -> Self {
        FrameCounter {
            disable_irq: true,
            sequence_mode: true,
        }
    }

    fn update(&mut self, data: u8) {
        self.disable_irq = data & 0x40 != 0;
        self.sequence_mode = data & 0x80 != 0;
    }

    fn irq(&self) -> bool {
        !self.disable_irq
    }

    fn mode(&self) -> u8 {
        if self.sequence_mode { 5 } else { 4 }
    }
}








struct StatusRegister {
    enable_ch1: bool,
    enable_ch2: bool,
    enable_ch3: bool,
    enable_ch4: bool,
    enable_ch5: bool,
    enable_frame_irq: bool,
    enable_dmc_irq: bool,
}

impl StatusRegister {
    fn new() -> Self {
        StatusRegister { enable_ch1: false, enable_ch2: false, enable_ch3: false, enable_ch4: false, enable_ch5: false, enable_frame_irq: false, enable_dmc_irq: false }
    }

    fn get(&self) -> u8 {
        let mut data = 0;
        if self.enable_ch1 { data |= 0x01 }
        if self.enable_ch2 { data |= 0x02 }
        if self.enable_ch3 { data |= 0x04 }
        if self.enable_ch4 { data |= 0x08 }
        if self.enable_ch5 { data |= 0x10 }
        if self.enable_frame_irq { data |= 0x40 }
        if self.enable_dmc_irq { data |= 0x80 }
        data
    }

    fn update(&mut self, data: u8) {
        self.enable_ch1 = data & 0x01 != 0;
        self.enable_ch2 = data & 0x02 != 0;
        self.enable_ch3 = data & 0x04 != 0;
        self.enable_ch4 = data & 0x08 != 0;
        self.enable_ch5 = data & 0x10 != 0;
    }
}
