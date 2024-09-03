#include "common.h"
#include <SDL2/SDL.h>

#define CPU_HERTZ (1789773.0)

typedef struct {
    float duty;
    float volume;
    float hertz;
} SquareWave;

SquareWave square1;

void square1_callback(void *userdata, Uint8 *stream, int len) {
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
    SDL_AudioSpec desired, obtained;
    SDL_zero(desired);

    square1.duty = 0.0;
    square1.volume = 0.0;
    square1.hertz = 0.0;

    desired.callback = square1_callback;
    desired.channels = 1;
    desired.format = AUDIO_F32;
    desired.freq = 44100;
    desired.samples = 4096;
    desired.userdata = &square1;

    if(SDL_OpenAudio(&desired, &obtained) < 0) {
        error("SDL_OpenAudio() failed\n");
    }
    SDL_PauseAudio(0);
}

void init_apu(void) {
    init_channel1();
}

typedef struct {
    unsigned char register1;
    unsigned char register2;
    unsigned char register3;
    unsigned char register4;
} Channel1;

Channel1 channel1;

void write_square1(unsigned short address, unsigned char value) {
    if(address == 0x4000) {
        channel1.register1 = value;
        switch((channel1.register1 >> 6) & 0x03) {
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
                error("Invalid duty of square1\n");
        }
        square1.volume = (channel1.register1 & 0x0f) / 15.0;
    } else if(address == 0x4001) {
        channel1.register2 = value;
    } else if(address == 0x4002) {
        channel1.register3 = value;
        square1.hertz = CPU_HERTZ / (16 * ((channel1.register3 + ((channel1.register4 & 0x07) << 8)) + 1));
    } else if(address == 0x4003) {
        channel1.register4 = value;
        square1.hertz = CPU_HERTZ / (16 * ((channel1.register3 + ((channel1.register4 & 0x07) << 8)) + 1));
    } else {
        error("Invalid write to square1\n");
    }
}
