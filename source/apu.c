#include "common.h"
#include <SDL2/SDL.h>

#define CPU_HERTZ (1789773.0)

typedef struct {
    float duty;
    float volume;
    float hertz;
} SquareWave;

SquareWave square1, square2;

void square_callback(void *userdata, Uint8 *stream, int len) {
    static float phase = 0.0;
    static float frequency = 44100.0;
    SquareWave *note = userdata;
    float *buffer = (float*)stream;
    int sample_rate = len / sizeof(float);
    for(int i = 0; i < sample_rate; i++) {
        if(phase < note->duty) {
            buffer[i] = note->volume;
        } else {
            buffer[i] = -note->volume;
        }
        phase += note->hertz / frequency;
        phase -= (int)phase;
    }
}

void init_channel1(void) {
    SDL_AudioSpec desired;
    SDL_zero(desired);

    square1.duty = 0.0;
    square1.volume = 0.0;
    square1.hertz = 0.0;

    desired.callback = square_callback;
    desired.channels = 1;
    desired.format = AUDIO_F32;
    desired.freq = 44100;
    desired.samples = 4096;
    desired.userdata = &square1;

    SDL_AudioDeviceID device = SDL_OpenAudioDevice(NULL, 0, &desired, NULL, 0);
    SDL_PauseAudioDevice(device, 0);
}

void init_channel2(void) {
    SDL_AudioSpec desired;
    SDL_zero(desired);

    square2.duty = 0.0;
    square2.volume = 0.0;
    square2.hertz = 0.0;

    desired.callback = square_callback;
    desired.channels = 1;
    desired.format = AUDIO_F32;
    desired.freq = 44100;
    desired.samples = 4096;
    desired.userdata = &square2;

    SDL_AudioDeviceID device = SDL_OpenAudioDevice(NULL, 0, &desired, NULL, 0);
    SDL_PauseAudioDevice(device, 0);
}

void write_square1(unsigned short address, unsigned char value) {
    static unsigned char frequency_low, frequency_high;
    if(address == 0x4000) {
        switch((value >> 6) & 0x03) {
            case 0:
                square1.duty = 0.125;
                break;
            case 1:
                square1.duty = 0.25;
                break;
            case 2:
                square1.duty = 0.5;
                break;
            case 3:
                square1.duty = 0.75;
                break;
            default:
                error("Invalid duty to square1\n");
        }
        square1.volume = (value & 0x0f) / 15.0;
    } else if(address == 0x4001) {

    } else if(address == 0x4002) {
        frequency_low = value;
        square1.hertz = CPU_HERTZ / (16 * ((frequency_low + (frequency_high << 8)) + 1));
    } else if(address == 0x4003) {
        frequency_high = value & 0x07;
        square1.hertz = CPU_HERTZ / (16 * ((frequency_low + (frequency_high << 8)) + 1));
    } else {
        error("Invalid write to square1\n");
    }
}

void write_square2(unsigned short address, unsigned char value) {
    static unsigned char frequency_low, frequency_high;
    if(address == 0x4004) {
        switch((value >> 6) & 0x03) {
            case 0:
                square2.duty = 0.125;
                break;
            case 1:
                square2.duty = 0.25;
                break;
            case 2:
                square2.duty = 0.5;
                break;
            case 3:
                square2.duty = 0.75;
                break;
            default:
                error("Invalid duty to square2\n");
        }
        square2.volume = (value & 0x0f) / 15.0;
    } else if(address == 0x4005) {

    } else if(address == 0x4006) {
        frequency_low = value;
        square2.hertz = CPU_HERTZ / (16 * ((frequency_low + (frequency_high << 8)) + 1));
    } else if(address == 0x4007) {
        frequency_high = value & 0x07;
        square2.hertz = CPU_HERTZ / (16 * ((frequency_low + (frequency_high << 8)) + 1));
    } else {
        error("Invalid write to square2\n");
    }
}

typedef struct {
    float hertz;
} TriangleWave;

TriangleWave triangle;

void triangle_callback(void *userdata, Uint8 *stream, int len) {
    static float phase = 0.0;
    static float frequency = 44100.0;
    TriangleWave *note = userdata;
    float *buffer = (float*)stream;
    int sample_rate = len / sizeof(float);
    for(int i = 0; i < sample_rate; i++) {
        if(phase < 0.5) {
            buffer[i] = phase;
        } else {
            buffer[i] = 1 - phase;
        }
        buffer[i] = (buffer[i] - 0.25) * 4;
        phase += note->hertz / frequency;
        phase -= (int)phase;
    }
}

void init_channel3(void) {
    SDL_AudioSpec desired;
    SDL_zero(desired);

    triangle.hertz = 0.0;

    desired.callback = triangle_callback;
    desired.channels = 1;
    desired.format = AUDIO_F32;
    desired.freq = 44100;
    desired.samples = 4096;
    desired.userdata = &triangle;

    SDL_AudioDeviceID device = SDL_OpenAudioDevice(NULL, 0, &desired, NULL, 0);
    SDL_PauseAudioDevice(device, 0);
}

void write_triangle(unsigned short address, unsigned char value) {
    static unsigned char frequency_low, frequency_high;
    if(address == 0x4008) {

    } else if(address == 0x400a) {
        frequency_low = value;
        triangle.hertz = CPU_HERTZ / (16 * ((frequency_low + (frequency_high << 8)) + 1));
    } else if(address == 0x400b) {
        frequency_high = value & 0x07;
        triangle.hertz = CPU_HERTZ / (16 * ((frequency_low + (frequency_high << 8)) + 1));
    } else {
        error("Invalid write to triangle\n");
    }
}

typedef struct {
    float volume;
    float hertz;
    unsigned short shift_register;
    unsigned char bit;
} Noise;

Noise noise;

void noise_callback(void *userdata, Uint8 *stream, int len) {
    static float phase = 0.0;
    static float frequency = 44100.0;
    Noise *note = userdata;
    float *buffer = (float*)stream;
    int sample_rate = len / sizeof(float);
    for(int i = 0; i < sample_rate; i++) {
        buffer[i] = (!(note->shift_register & 0x01)) * note->volume;
        float last_phase = phase;
        phase += note->hertz / frequency;
        phase -= (int)phase;
        if(last_phase > phase) {
            bool bit0 = note->shift_register & 0x01;
            bool bitn = (note->shift_register >> note->bit) & 0x01;
            bool feedback = bit0 ^ bitn;
            note->shift_register >>= 1;
            note->shift_register = (note->shift_register & 0xbfff) + (feedback << 14);
        }
    }
}

void init_channel4(void) {
    SDL_AudioSpec desired;
    SDL_zero(desired);

    noise.volume = 0.0;
    noise.hertz = 0.0;
    noise.shift_register = 1;
    noise.bit = 1;

    desired.callback = noise_callback;
    desired.channels = 1;
    desired.format = AUDIO_F32;
    desired.freq = 44100;
    desired.samples = 4096;
    desired.userdata = &noise;

    SDL_AudioDeviceID device = SDL_OpenAudioDevice(NULL, 0, &desired, NULL, 0);
    SDL_PauseAudioDevice(device, 0);
}

void write_noise(unsigned short address, unsigned char value) {
    if(address == 0x400c) {
        noise.volume = value & 0x0f;
    } else if(address == 0x400e) {
        noise.bit = (value & 0x80) == 0 ? 1 : 6;
    } else if(address == 0x400f) {

    } else {
        error("Invalid write to noise\n");
    }
}

void init_apu(void) {
    init_channel1();
    init_channel2();
    init_channel3();
    init_channel4();
}
