mod bootrom;
mod hram;
mod peripherals;
mod cpu;
mod ppu;
mod lcd;
mod gameboy;

use bootrom::Bootrom;
use gameboy::GameBoy;
use std::fs;

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
pub const LCD_PIXELS: usize = LCD_WIDTH * LCD_HEIGHT;

fn main() {
    println!("Starting GameBoy emulator...");
    
    // dmg_bootrom.binを読み込む
    let bootrom_data = fs::read("dmg_bootrom.bin")
        .expect("Failed to read dmg_bootrom.bin");
    println!("Loaded bootrom: {} bytes", bootrom_data.len());
    
    // Bootromを作成
    let bootrom = Bootrom::new(bootrom_data.into_boxed_slice());
    println!("Created Bootrom");
    
    // GameBoyを作成
    println!("Creating GameBoy...");
    let mut gameboy = GameBoy::new(bootrom);
    println!("GameBoy created, starting emulation...");
    
    // GameBoyを実行
    gameboy.run();
    println!("Emulation ended.");
}
