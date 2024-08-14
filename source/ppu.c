#include "common.h"
#include <stdbool.h>
#include <gtk-3.0/gtk/gtk.h>

#define between(start, address, end) (start <= address && address <= end)
#define MIRROR_HORIZONTAL (0)
#define MIRROR_VERTICAL (1)
#define TILE_PIXEL_SIZE (8)
#define TILE_NUMBER_X (32)
#define TILE_NUMBER_Y (30)
#define PATTERN_BYTE_SIZE (16)
#define PATTERN_TABLE_BYTE_SIZE (PATTERN_BYTE_SIZE * 256)

extern ROM *rom;
extern unsigned char frame[BYTE_PER_PIXEL * SCREEN_PIXEL_WIDTH * SCREEN_PIXEL_HEIGHT];
extern GtkWidget *drawing_area;

void nmi(void);

// 0x2005と0x2006で共有されるアドレスラッチ
bool w;

unsigned int ppu_cycle, scanline;
unsigned char nametable[0x800];
unsigned char *nametable_top_left, *nametable_top_right, *nametable_bottom_left, *nametable_bottom_right;
unsigned char palette_table[0x20];

unsigned char color[] = {
    0x80, 0x80, 0x80, 0xA6, 0x3D, 0x00, 0xB0, 0x12, 0x00, 0x00, 0x96, 0x44, 0x5E, 0x00, 0xA1,
    0x28, 0x00, 0xC7, 0x00, 0x06, 0xBA, 0x00, 0x17, 0x8C, 0x2F, 0x00, 0x5C, 0x00, 0x45, 0x10,
    0x00, 0x4A, 0x05, 0x2E, 0x47, 0x00, 0x66, 0x41, 0x00, 0x00, 0x00, 0x00, 0x05, 0x05, 0x05,
    0x05, 0x05, 0x05, 0xC7, 0xC7, 0xC7, 0xFF, 0x77, 0x00, 0x55, 0xFF, 0x21, 0xFA, 0x37, 0x82,
    0xB5, 0x2F, 0xEB, 0x50, 0x29, 0xFF, 0x00, 0x22, 0xFF, 0x32, 0x00, 0xD6, 0x00, 0x62, 0xC4,
    0x00, 0x80, 0x35, 0x00, 0x8F, 0x05, 0x55, 0x8A, 0x00, 0x99, 0xCC, 0x00, 0x21, 0x21, 0x21,
    0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0xFF, 0xFF, 0xFF, 0xD7, 0xFF, 0x0F, 0xFF, 0xA2, 0x69,
    0xFF, 0x80, 0xD4, 0xF3, 0x45, 0xFF, 0x8B, 0x61, 0xFF, 0x88, 0x33, 0xFF, 0x12, 0x9C, 0xFF,
    0x20, 0xBC, 0xFA, 0x0E, 0xE3, 0x9F, 0x35, 0xF0, 0x2B, 0xF0, 0xA4, 0x0C, 0xFF, 0xFB, 0x05,
    0x5E, 0x5E, 0x5E, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0xFF, 0xFF, 0xFF, 0xFF, 0xFC, 0xA6,
    0xFF, 0xEC, 0xB3, 0xEB, 0xAB, 0xDA, 0xF9, 0xA8, 0xFF, 0xAB, 0xB3, 0xFF, 0xB0, 0xD2, 0xFF,
    0xA6, 0xEF, 0xFF, 0x9C, 0xF7, 0xFF, 0x95, 0xE8, 0xD7, 0xED, 0xAF, 0xA6, 0xDA, 0xF2, 0xA2,
    0xFC, 0xFF, 0x99, 0xDD, 0xDD, 0xDD, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11
};

// 0x2000 (Write)
typedef struct {
    // 0 => 0x2000, 1 => 0x2400, 2 => 0x2800, 3 => 0x2c00
    // 実質的にスクロールの最上位ビットと捉えることができる (xスクロール += 256 * ビット0、yスクロール += 240 * ビット1)
    unsigned int base_nametable_address;
    // 0 => add 1, 1 => add 32 (0x2007の読み書きにおけるアドレス増加量)
    bool increment_address;
    // 0 => 0x0000, 1 => 0x1000 (8*16モードでは無視され、OAMのバイト1を参照する)
    bool sprite_pattern_table_address;
    // 0 => 0x0000, 1 => 0x1000
    bool background_pattern_table_address;
    // 0 => 8*8モード, 1 => 8*16モード
    bool sprite_size;
    // 0 => off, 1 => on (0x2002のin_vblankがtrueの場合、垂直ブランキング期間にNMIを発生させる)
    bool generate_nmi;
} PPU_Control;

PPU_Control ppu_control;

// 0x2001 (Write)
// render_leftmost_backgroundとrender_leftmost_spriteはピクセルマスクである
typedef struct {
    // 0 => 通常色, 1 => グレースケール
    // 画面の表示では、0x3f00-0x3fffからの読み取りは0x30とのAND演算で実装される
    bool gray_scale;
    // 0 => 非表示, 1 => 左端8ピクセルに背景を表示
    bool render_leftmost_background;
    // 0 => 非表示, 1 => 左端8ピクセルにスプライトを表示
    bool render_leftmost_sprite;
    bool render_background;
    bool render_sprite;
} PPU_Mask;

PPU_Mask ppu_mask;

void write_ppu_mask(unsigned char value) {
    ppu_mask.gray_scale = (value >> 0) & 0x01;
    ppu_mask.render_leftmost_background = (value >> 1) & 0x01;
    ppu_mask.render_leftmost_sprite = (value >> 2) & 0x01;
    ppu_mask.render_background = (value >> 3) & 0x01;
    ppu_mask.render_sprite = (value >> 4) & 0x01;
}

// 0x2002 (Read)
// sprite_overflowとsprite0_hitはフレーム開始時にクリアされる
typedef struct {
    // スキャンライン上に8個を超えるスプライトを表示しようとするとセットされる
    bool sprite_overflow;
    // スプライト0と背景の不透明ピクセルが重なった場合にセットされる
    // 0x2001に関連し、背景とスプライトの少なくとも一方が非表示の領域ではセットされない
    bool sprite0_hit;
    // 垂直ブランキング期間(241行目)にセットされる
    bool in_vblank;
} PPU_Status;

PPU_Status ppu_status;

void write_ppu_control(unsigned char value) {
    bool old_generate_nmi = ppu_control.generate_nmi;
    ppu_control.base_nametable_address = (value >> 0) & 0x03;
    ppu_control.increment_address = (value >> 2) & 0x01;
    ppu_control.sprite_pattern_table_address = (value >> 3) & 0x01;
    ppu_control.background_pattern_table_address = (value >> 4) & 0x01;
    ppu_control.sprite_size = (value >> 5) & 0x01;
    ppu_control.generate_nmi = (value >> 7) & 0x01;
    if(old_generate_nmi == false && ppu_control.generate_nmi == true && ppu_status.in_vblank == true) {
        nmi();
    }
}

unsigned char read_ppu_status(void) {
    unsigned char value = 0;
    if(ppu_status.sprite_overflow) value |= 0x20;
    if(ppu_status.sprite0_hit) value |= 0x40;
    if(ppu_status.in_vblank) value |= 0x80;
    ppu_status.in_vblank = w = false;
    return value;
}

// 0x2003 (Write)
unsigned char oam_address;

void write_oam_address(unsigned char value) {
    oam_address = value;
}

// 0x2004 (Read / Write)
unsigned char oam_data[256];

unsigned char read_oam_data(void) {
    return oam_data[oam_address];
}

void write_oam_data(unsigned char value) {
    oam_data[oam_address++] = value;
}

// 0x2005 (Write)
unsigned char scroll_x;
unsigned char scroll_y;

void write_ppu_scroll(unsigned char value) {
    if(w == false) {
        scroll_x = value;
    } else {
        scroll_y = value;
    }
    w = !w;
}

// 0x2006 (Write)
unsigned short ppu_address;

void write_ppu_address(unsigned char value) {
    if(w == false) {
        ppu_address = (ppu_address & 0x00ff) + (value << 8);
    } else {
        ppu_address = (ppu_address & 0xff00) + value;
    }
    w = !w;
}

// 0x2007 (Read / Write)
// PPUのメモリマップ (0x2006と0x2007を使用し、CPU経由のアクセスも可能)
// 0x0000-0x0fff パターンテーブル0
// 0x1000-0x1fff パターンテーブル1
// 0x2000-0x23ff ネームテーブル0
// 0x2400-0x27ff ネームテーブル1
// 0x2800-0x2bff ネームテーブル2
// 0x2c00-0x2fff ネームテーブル3
// 0x3000-0x3eff 0x2000-0x2effのミラー
// 0x3f00-0x3f1f パレット
// 0x3f20-0x3fff 0x3f00-0x3f1fのミラー
// 0x4000-0xffff 0x0000-0x3fffのミラー
unsigned char buffer;

unsigned char read_ppu_data(void) {
    unsigned char value = buffer;
    ppu_address &= 0x3fff;
    if(between(0x0000, ppu_address, 0x1fff)) {
        buffer = rom->character_rom[ppu_address];
    } else if(between(0x2000, ppu_address, 0x3eff)) {
        switch((ppu_address >> 10) & 0x03) {
            case 0:
                buffer = nametable_top_left[ppu_address & 0x3ff];
                break;
            case 1:
                buffer = nametable_top_right[ppu_address & 0x3ff];
                break;
            case 2:
                buffer = nametable_bottom_left[ppu_address & 0x3ff];
                break;
            case 3:
                buffer = nametable_bottom_right[ppu_address & 0x3ff];
                break;
            default:
                error("Invalid nametable read\n");
        }
    } else if(between(0x3f00, ppu_address, 0x3fff)) {
        unsigned int nibble = ppu_address & 0xf;
        if((ppu_address & 0x10) != 0 && (nibble == 0x0 || nibble == 0x4 || nibble == 0x8 || nibble == 0xc)) {
            value = buffer = palette_table[(ppu_address - 0x10) & 0x1f];
        } else {
            value = buffer = palette_table[ppu_address & 0x1f];
        }
        if(ppu_mask.gray_scale) {
            value = buffer = buffer & 0x30;
        }
    } else {
        error("Invalid ppu read 0x%04X\n", ppu_address);
    }
    ppu_address += ppu_control.increment_address ? 32 : 1;
    return value;
}

void write_ppu_data(unsigned char value) {
    ppu_address &= 0x3fff;
    if(between(0x0000, ppu_address, 0x1fff)) {
        // error("Unsupported CHR-ROM write\n");
    } else if(between(0x2000, ppu_address, 0x3eff)) {
        switch((ppu_address >> 10) & 0x03) {
            case 0:
                nametable_top_left[ppu_address & 0x3ff] = value;
                break;
            case 1:
                nametable_top_right[ppu_address & 0x3ff] = value;
                break;
            case 2:
                nametable_bottom_left[ppu_address & 0x3ff] = value;
                break;
            case 3:
                nametable_bottom_right[ppu_address & 0x3ff] = value;
                break;
            default:
                error("Invalid nametable write\n");
        }
    } else if(between(0x3f00, ppu_address, 0x3fff)) {
        unsigned int nibble = ppu_address & 0xf;
        if((ppu_address & 0x10) != 0 && (nibble == 0x0 || nibble == 0x4 || nibble == 0x8 || nibble == 0xc)) {
            palette_table[(ppu_address - 0x10) & 0x1f] = value;
        } else {
            palette_table[ppu_address & 0x1f] = value;
        }
    } else {
        error("Invalid ppu write 0x%04X\n", ppu_address);
    }
    ppu_address += ppu_control.increment_address ? 32 : 1;
}

// FIXME
bool is_sprite0_hit(void) {
    return oam_data[0] == scanline && oam_data[3] <= ppu_cycle && ppu_mask.render_background && ppu_mask.render_sprite;
}

// FIXME
void tick_ppu(unsigned int cycle) {
    ppu_cycle += cycle;
    if(ppu_cycle >= 341) {
        if(is_sprite0_hit()) {
            ppu_status.sprite0_hit = true;
        }
        ppu_cycle -= 341;
        scanline += 1;
        if(scanline == 241) {
            gtk_widget_queue_draw(drawing_area);
            ppu_status.in_vblank = true;
            if(ppu_control.generate_nmi) {
                nmi();
            }
        } else if(scanline == 262) {
            scanline = 0;
            ppu_status.sprite_overflow = false;
            ppu_status.sprite0_hit = false;
            ppu_status.in_vblank = false;
        }
    }
}

void init_ppu(void) {
    w = false;
    write_ppu_control(0);
    write_ppu_mask(0);
    oam_address = 0;
    scroll_x = scroll_y = 0;
    ppu_address = 0;
    buffer = 0;

    if(rom->mirroring == MIRROR_HORIZONTAL) {
        switch(ppu_control.base_nametable_address) {
            case 0: case 1:
                nametable_top_left = nametable_top_right = nametable;
                nametable_bottom_left = nametable_bottom_right = nametable + 0x400;
                break;
            case 2: case 3:
                nametable_top_left = nametable_top_right = nametable + 0x400;
                nametable_bottom_left = nametable_bottom_right = nametable;
                break;
        }
    } else if(rom->mirroring == MIRROR_VERTICAL) {
        switch(ppu_control.base_nametable_address) {
            case 0: case 2:
                nametable_top_left = nametable_bottom_left = nametable;
                nametable_top_right = nametable_bottom_right = nametable + 0x400;
                break;
            case 1: case 3:
                nametable_top_left = nametable_bottom_left = nametable + 0x400;
                nametable_top_right = nametable_bottom_right = nametable;
                break;
        }
    } else {
        error("Unsupported mirroring\n");
    }
}

unsigned char *create_palette(int palette_index) {
    static unsigned char palette[4];
    palette[0] = palette_table[0];
    for(int i = 1; i <= 3; i++) {
        palette[i] = palette_table[4 * palette_index + i];
    }
    return palette;
}

void render_pixel(int px, int py, unsigned char *c) {
    unsigned int *p = (unsigned int*)frame + BLOCK_PIXEL_SIZE * (px + SCREEN_PIXEL_WIDTH * py);
    unsigned int rgb = *(unsigned int*)c;
    for(int i = 0; i < BLOCK_PIXEL_SIZE; i++, p += SCREEN_PIXEL_WIDTH) {
        for(int j = 0; j < BLOCK_PIXEL_SIZE; j++) {
            p[j] = rgb;
        }
    }
}

void render_nametable(int base_px, int base_py, unsigned char *nametable) {
    unsigned char *pattern_table = rom->character_rom + PATTERN_TABLE_BYTE_SIZE * ppu_control.background_pattern_table_address;
    int sx = ppu_mask.render_leftmost_background ? 0 : 8;
    int stx, sty;
    for(stx = 0; base_px + TILE_PIXEL_SIZE * (stx + 1) - 1 < 0; stx++);
    for(sty = 0; base_py + TILE_PIXEL_SIZE * (sty + 1) - 1 < 0; sty++);
    for(int ty = sty; ty < TILE_NUMBER_Y && base_py + TILE_PIXEL_SIZE * ty < SCREEN_BLOCK_HEIGHT; ty++) {
        for(int tx = stx; tx < TILE_NUMBER_X && base_px + TILE_PIXEL_SIZE * tx < SCREEN_BLOCK_WIDTH; tx++) {
            unsigned char attribute = nametable[0x3c0 + (tx / 4) + 8 * (ty / 4)];
            unsigned char *palette = create_palette((attribute >> (2 * ((tx / 2) % 2) + 4 * ((ty / 2) % 2))) & 0x03);
            unsigned char *pattern = pattern_table + PATTERN_BYTE_SIZE * nametable[tx + TILE_NUMBER_X * ty];
            for(int py = 0; py < TILE_PIXEL_SIZE; py++) {
                unsigned char pattern_low = pattern[py];
                unsigned char pattern_high = pattern[py + 8];
                for(int px = 0; px < TILE_PIXEL_SIZE; px++) {
                    int x = base_px + TILE_PIXEL_SIZE * tx + px;
                    int y = base_py + TILE_PIXEL_SIZE * ty + py;
                    int color_index = ((pattern_low >> (7 - px)) & 1) + ((pattern_high >> (7 - px)) & 1) * 2;
                    if(between(sx, x, SCREEN_BLOCK_WIDTH) && between(0, y, SCREEN_BLOCK_HEIGHT)) {
                        render_pixel(x, y, color + 3 * palette[color_index]);
                    }
                }
            }
        }
    }
}

void render_background(void) {
    render_nametable(-scroll_x, -scroll_y, nametable_top_left);
    render_nametable(SCREEN_BLOCK_WIDTH - scroll_x, -scroll_y, nametable_top_right);
    render_nametable(-scroll_x, SCREEN_BLOCK_HEIGHT - scroll_y, nametable_bottom_left);
    render_nametable(SCREEN_BLOCK_WIDTH - scroll_x, SCREEN_BLOCK_HEIGHT - scroll_y, nametable_bottom_right);
}

void render(void) {
    if(ppu_mask.render_background) {
        render_background();
    }
}
