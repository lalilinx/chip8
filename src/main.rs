use std::time::{Duration, SystemTime};
fn main() {
    println!("init");
}

const FPS: f32 = 16.67;

struct Memory {
    memory: [u8; 4096],
}

impl Memory {
    fn read(&self) {}
    fn write(&mut self, address: u16, data: u8) {}
}

// struct FrameBuffer {
//     buffer: []
// }

struct Chip8 {
    registers: [u8; 16],
    i: u16,
    pc: u16,          // Program counter
    sp: u16,          // Stack pointer
    stack: [u16; 16], // Stack for storing return addresses
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],

    memory: Memory,
    // frame: [u8; ]
}

fn nibble(value: &u16, n: u8) -> u8 {
    ((value >> (n * 4)) & 0xF) as u8
}

impl Chip8 {
    fn cycle(&mut self) {
        loop {
            let now = SystemTime::now();

            match now.elapsed() {
                Ok(elapsed) => {
                    println!("{}", elapsed.as_millis());
                }
                Err(e) => {
                    println!("SystemTimeError difference: {:?}", e.duration());
                }
            }
        }
    }
    fn fetch(&mut self) {}
    fn decode_and_excute(&mut self, inst: u16) {
        let opcode: u16 = (inst >> 12) & 0xF;
        match opcode {
            0x0 => {
                if inst == 0xE0 {
                    // clear frame buffer to 0
                }
            }
            0x1 => {
                let index: u16 = inst << 4;
                self.pc = index;
            }
            0x2 => {
                let index: u16 = inst << 4;
                match self.push(self.pc) {
                    Err(e) => {
                        print!("{}", e)
                    }
                    _ => (),
                }
                self.pc = index;
            }
            0x3 => {
                let x: u8 = nibble(&inst, 2);
                let vx = self.get_register_data(x);
                let kk: u8 = inst as u8;
                if vx == kk {
                    self.pc += 2;
                }
            }
            0x4 => {
                let x: u8 = nibble(&inst, 2);
                let vx = self.get_register_data(x);
                let kk: u8 = inst as u8;
                if vx != kk {
                    self.pc += 2;
                }
            }
            0x5 => {
                let x: u8 = nibble(&inst, 2);
                let vx = self.get_register_data(x);
                let y: u8 = nibble(&inst, 1);
                let vy = self.get_register_data(y);
                if vx == vy {
                    self.pc += 2;
                }
            }
            0x6 => {
                let x: u8 = nibble(&inst, 2);
                let kk: u8 = inst as u8;
                self.register(x, kk);
            }
            0x7 => {
                let x: u8 = nibble(&inst, 2);
                let vx = self.get_register_data(x);
                let kk: u8 = inst as u8;
                self.register(x, vx + kk);
            }
            0x8 => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(x);
                let y: u8 = nibble(&inst, 1);
                let vy: u8 = self.get_register_data(y);

                let indic: u8 = nibble(&inst, 0);
                match indic {
                    0x0 => {
                        self.register(x, vy);
                    }
                    0x1 => {
                        self.register(x, vx | vy);
                    }
                    0x2 => {
                        self.register(x, vx & vy);
                    }
                    0x3 => {
                        self.register(x, vx ^ vy);
                    }
                    0x4 => {
                        let (sum, is_overflow) = vx.overflowing_add(vy);
                        if is_overflow {
                            self.register(0xF, 0x1);
                            self.register(x, sum);
                            return;
                        }
                        self.register(0xF, 0x0);
                    }
                    0x5 => {
                        if vx > vy {
                            self.register(0xF, 0x1);
                            self.register(x, vx - vy);
                            return;
                        }
                        self.register(0xF, 0x0);
                    }
                    0x6 => {
                        let lsb: u8 = vx & 0x1;
                        if lsb == 0x1 {
                            self.register(0xF, lsb);
                            self.register(x, vx >> 0x1);
                        }
                        self.register(0xF, 0x0);
                    }
                    0x7 => {
                        if vy > vx {
                            self.register(0xF, 0x1);
                            self.register(x, vy - vx);
                        }
                        self.register(0xF, 0x0);
                    }
                    0xE => {
                        let msb: u8 = (vx >> 7) & 0x1;
                        if msb == 0x1 {
                            self.register(0xF, 0x1);
                            self.register(x, vx << 1);
                        }
                        self.register(0xF, 0x0);
                    }
                    _ => (),
                }
            }
            0x9 => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(x);
                let y: u8 = nibble(&inst, 1);
                let vy: u8 = self.get_register_data(y);

                if vx != vy {
                    self.pc += 2
                }
            }
            0xA => {}
            0xB => {}
            0xC => {}
            0xD => {}
            0xE => {}
            0xF => {}
            _ => println!("overflow"),
        }
    }

    fn get_register_data(&self, regi: u8) -> u8 {
        self.registers[regi as usize]
    }
    fn register(&mut self, regi: u8, data: u8) -> Result<(), &'static str> {
        if regi as usize >= self.registers.len() {
            return Err("Overflow");
        }
        self.registers[regi as usize] = data;
        Ok(())
    }

    fn push(&mut self, data: u16) -> Result<(), &'static str> {
        if self.sp as usize >= self.stack.len() {
            return Err("Stack Overflow");
        }
        self.stack[self.sp as usize] = data;
        self.sp += 1;
        Ok(())
    }

    fn pop(&mut self) -> Option<u16> {
        if self.sp == 0 {
            return None; // Stack underflow
        }
        self.sp -= 1;
        Some(self.stack[self.sp as usize])
    }
}
