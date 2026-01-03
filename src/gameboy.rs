use std::time;

use crate::{bootrom::Bootrom, cpu::Cpu, lcd::LCD, peripherals::Peripherals};

pub const CPU_CLOCK_HZ: u128 = 4_194_304;
pub const M_CYCLE_CLOCK: u128 = 4;
const M_CYCLE_NANOS: u128 = M_CYCLE_CLOCK * 1_000_000_000 / CPU_CLOCK_HZ;

pub struct GameBoy {
    pub cpu: Cpu,
    pub peripherals: Peripherals,
    pub lcd: LCD,
    sdl: sdl2::Sdl,
}

impl GameBoy {
    pub fn new(bootrom: Bootrom) -> Self {
        println!("Initializing SDL2...");
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        println!("SDL2 initialized");
        
        println!("Creating LCD window...");
        let lcd = LCD::new(&sdl, 4);
        println!("LCD window created");
        
        let peripherals = Peripherals::new(bootrom);
        let cpu = Cpu::new();
        Self {
            cpu,
            peripherals,
            lcd,
            sdl,
        }
    }
    pub fn run(&mut self) {
        println!("Starting main loop...");
        let mut event_pump = self.sdl.event_pump().expect("Failed to get event pump");
        let time = time::Instant::now();
        let mut elapsed = 0;
        let mut frame_count = 0;
        let mut iteration = 0;
        let mut cpu_cycles = 0;
        'running: loop {
            iteration += 1;
            if iteration % 100000 == 0 {
                println!("Loop iteration: {}, frames: {}, cpu_cycles: {}", iteration, frame_count, cpu_cycles);
            }
            
            // SDL2のイベントを処理
            for event in event_pump.poll_iter() {
                use sdl2::event::Event;
                match event {
                    Event::Quit { .. } => {
                        println!("Quit event received");
                        break 'running;
                    },
                    _ => {}
                }
            }
            
            let e = time.elapsed().as_nanos();
            let cycles = (e - elapsed) / M_CYCLE_NANOS;
            
            if iteration == 1 {
                println!("First cycle count: {}, M_CYCLE_NANOS: {}", cycles, M_CYCLE_NANOS);
            }
            
            if cycles > 0 {
                for i in 0..cycles {
                    cpu_cycles += 1;
                    self.cpu.emulate_cycle(&mut self.peripherals);
                    if self.peripherals.ppu.emulate_cycle() {
                        frame_count += 1;
                        println!("Frame {} completed!", frame_count);
                        self.lcd.draw(self.peripherals.ppu.pixel_buffer());
                    }
                    elapsed += M_CYCLE_NANOS;
                }
            }
            
            // CPU時間を少し解放
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        println!("Loop ended");
    }
}