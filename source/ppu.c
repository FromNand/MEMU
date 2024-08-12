#include "common.h"
#include <stdbool.h>

#define between(start, address, end) (start <= address && address <= end)

extern ROM *rom;

// 0x2005と0x2006で使用するアドレスラッチ
bool w;

unsigned char nametable[0x800];
unsigned char *nametable_top_left, *nametable_top_right, *nametable_bottom_left, *nametable_bottom_right;
unsigned char palette_table[0x20];

unsigned char color[] = {
    0x80, 0x80, 0x80, 0x00, 0x3D, 0xA6, 0x00, 0x12, 0xB0, 0x44, 0x00, 0x96, 0xA1, 0x00, 0x5E,
    0xC7, 0x00, 0x28, 0xBA, 0x06, 0x00, 0x8C, 0x17, 0x00, 0x5C, 0x2F, 0x00, 0x10, 0x45, 0x00,
    0x05, 0x4A, 0x00, 0x00, 0x47, 0x2E, 0x00, 0x41, 0x66, 0x00, 0x00, 0x00, 0x05, 0x05, 0x05,
    0x05, 0x05, 0x05, 0xC7, 0xC7, 0xC7, 0x00, 0x77, 0xFF, 0x21, 0x55, 0xFF, 0x82, 0x37, 0xFA,
    0xEB, 0x2F, 0xB5, 0xFF, 0x29, 0x50, 0xFF, 0x22, 0x00, 0xD6, 0x32, 0x00, 0xC4, 0x62, 0x00,
    0x35, 0x80, 0x00, 0x05, 0x8F, 0x00, 0x00, 0x8A, 0x55, 0x00, 0x99, 0xCC, 0x21, 0x21, 0x21,
    0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0xFF, 0xFF, 0xFF, 0x0F, 0xD7, 0xFF, 0x69, 0xA2, 0xFF,
    0xD4, 0x80, 0xFF, 0xFF, 0x45, 0xF3, 0xFF, 0x61, 0x8B, 0xFF, 0x88, 0x33, 0xFF, 0x9C, 0x12,
    0xFA, 0xBC, 0x20, 0x9F, 0xE3, 0x0E, 0x2B, 0xF0, 0x35, 0x0C, 0xF0, 0xA4, 0x05, 0xFB, 0xFF,
    0x5E, 0x5E, 0x5E, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0xFF, 0xFF, 0xFF, 0xA6, 0xFC, 0xFF,
    0xB3, 0xEC, 0xFF, 0xDA, 0xAB, 0xEB, 0xFF, 0xA8, 0xF9, 0xFF, 0xAB, 0xB3, 0xFF, 0xD2, 0xB0,
    0xFF, 0xEF, 0xA6, 0xFF, 0xF7, 0x9C, 0xD7, 0xE8, 0x95, 0xA6, 0xED, 0xAF, 0xA2, 0xF2, 0xDA,
    0x99, 0xFF, 0xFC, 0xDD, 0xDD, 0xDD, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11
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

void write_ppu_control(unsigned char value) {
    ppu_control.base_nametable_address = (value >> 0) & 0x03;
    ppu_control.increment_address = (value >> 2) & 0x01;
    ppu_control.sprite_pattern_table_address = (value >> 3) & 0x01;
    ppu_control.background_pattern_table_address = (value >> 4) & 0x01;
    ppu_control.sprite_size = (value >> 5) & 0x01;
    ppu_control.generate_nmi = (value >> 7) & 0x01;
}

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
        ppu_address |= value << 8;
    } else {
        ppu_address |= value;
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
        error("Unsupported CHR-ROM write\n");
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

void render(void) {

}
