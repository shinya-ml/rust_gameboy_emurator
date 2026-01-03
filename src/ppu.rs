use std::iter;

use crate::{LCD_PIXELS, LCD_WIDTH};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

// lcdc consts
const PPU_ENABLE: u8 = 1 << 7;
const WINDOW_TILE_MA: u8 = 1 << 6;
const WINDOW_ENABLE: u8 = 1 << 5;
const TILE_DATA_ADDRESSING_MODE: u8 = 1 << 4;
const BG_TILE_MAP: u8 = 1 << 3;
const SPRITE_SIZE: u8 = 1 << 2;
const SPRITE_ENABLE: u8 = 1 << 1;
const BG_WINDOW_ENABLE: u8 = 1 << 0;
// stat consts
const LYC_EQ_LY_INT: u8 = 1 << 6;
const OAM_SCAN_INT: u8 = 1 << 5;
const VBLANK_INT: u8 = 1 << 4;
const HBLANK_INT: u8 = 1 << 3;
const LYC_EQ_LY: u8 = 1 << 2;

pub struct Ppu {
    mode: Mode,
    // 0xFF40
    lcdc: u8,
    // 0xFF41
    stat: u8,
    // 0xFF42
    scy: u8,
    // 0xFF43
    scx: u8,
    // 0xFF44
    ly: u8,
    // 0xFF45
    lyc: u8,
    // 0xFF47
    bgp: u8,
    // 0xFF48
    obp0: u8,
    // 0xFF49
    obp1: u8,
    // 0xFF4A
    wy: u8,
    // 0xFF4B
    wx: u8,
    // 0x8000 - 0x9FFF
    vram: Box<[u8; 0x2000]>,
    // 0xFE00 - 0xFE9F
    oam: Box<[u8; 0xA0]>,
    pub buffer: Vec<u8>,
    cycles: u8,
}
impl Ppu {
    pub fn new() -> Self {
        Self {
            mode: Mode::OamScan,
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            vram: Box::new([0; 0x2000]),
            oam: Box::new([0; 0xA0]),
            buffer: vec![0; LCD_PIXELS*4],
            cycles: 20,
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => if self.mode == Mode::Drawing {
                0xFF
            } else {
                self.vram[addr as usize & 0x1FFF]
            },
            0xFE00..=0xFE9F => if self.mode == Mode::Drawing || self.mode == Mode::OamScan {
                0xFF
            } else {
                self.oam[addr as usize & 0xFF]
            },
            0xFF40 => self.lcdc,
            0xFF41 => 0x80 | self.stat | self.mode as u8,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => unreachable!(),
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000..=0x9FFF => if self.mode != Mode::Drawing {
                self.vram[addr as usize & 0x1FFF] = val;
            },
            0xFE00..=0xFE9F => if self.mode != Mode::Drawing && self.mode != Mode::OamScan {
                self.oam[addr as usize & 0xFF] = val;
            },
            0xFF40 => {
                if val & PPU_ENABLE > 0 && self.lcdc & PPU_ENABLE == 0 {
                    println!("PPU enabled! LCDC = 0x{:02X}", val);
                }
                self.lcdc = val;
            },
            0xFF41 => self.stat = (self.stat & LYC_EQ_LY) | (val & 0xF8),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {},
            0xFF45 => self.lyc = val,
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => unreachable!(),
        }
    }
    fn get_pixel_from_tile(&self, tile_idx: usize, row: u8, col:u8) -> u8 {
        let r = (row * 2) as usize;
        let c = (7 - col) as usize;
        let tile_addr = tile_idx << 4;
        let low = self.vram[(tile_addr | r) & 0x1FFF];
        let high = self.vram[(tile_addr | (r + 1)) & 0x1FFF];
        (((high >> c) & 1) << 1) | ((low >> c) & 1)
    }
    fn get_tile_idx_from_tile_map(&self, tile_map: bool, row: u8, col: u8) -> usize {
        let start_addr: usize = 0x1800 | ((tile_map as usize) << 10);
        let ret = self.vram[start_addr | (((row as usize) << 5) + col as usize) & 0x3FF];
        if self.lcdc & TILE_DATA_ADDRESSING_MODE > 0 {
            ret as usize
        } else {
            ((ret as i8 as i16) + 0x100) as usize
        }
    }
    fn render_bg(&mut self) {
        if self.lcdc & BG_WINDOW_ENABLE == 0 {
            return;
        }
        let y = self.ly.wrapping_add(self.scy);
        for i in 0..LCD_WIDTH {
            let x = (i as u8).wrapping_add(self.scx);
            let tile_idx = self.get_tile_idx_from_tile_map(
                self.lcdc & BG_TILE_MAP > 0,
                y >> 3,
                x >> 3,
            );

            let pixel = self.get_pixel_from_tile(tile_idx, y & 7, x & 7);

            self.buffer[LCD_WIDTH * self.ly as usize + i] =
                match (self.bgp >> (pixel << 1)) & 0b11 {
                    0b00 => 0xFF,
                    0b01 => 0xAA,
                    0b10 => 0x55,
                    _ => 0x00,
                }
        }
    }
    fn check_lyc_eq_ly(&mut self) {
        if self.ly == self.lyc {
            self.stat |= LYC_EQ_LY;
        } else {
            self.stat &= !LYC_EQ_LY;
        }
    }
    pub fn emulate_cycle(&mut self) -> bool {
        if self.lcdc & PPU_ENABLE == 0 {
            return false;
        }

        self.cycles -= 1;
        if self.cycles > 0 {
            return false;
        }

        let mut ret = false;
        match self.mode {
            Mode::HBlank => {
                self.ly += 1;
                if self.ly < 144 {
                    self.mode = Mode::OamScan;
                    self.cycles = 20;
                } else {
                    self.mode = Mode::VBlank;
                    self.cycles = 114;
                }
                self.check_lyc_eq_ly();
            },
            Mode::VBlank => {
                self.ly += 1;
                if self.ly > 153 {
                    ret = true;
                    self.ly = 0;
                    self.mode = Mode::OamScan;
                    self.cycles = 20;
                } else {
                    self.cycles = 114;
                }
                self.check_lyc_eq_ly();
            },
            Mode::OamScan => {
                self.mode = Mode::Drawing;
                self.cycles = 43;
            },
            Mode::Drawing => {
                self.render_bg();
                self.mode = Mode::HBlank;
                self.cycles = 51;
            }
        }
        ret
    }
    pub fn pixel_buffer(&self) -> Box<[u8]> {
        self.buffer.iter().flat_map(
            |&e| iter::repeat(e.into()).take(3)
        ).collect::<Box<[u8]>>()
    }
}
       



   