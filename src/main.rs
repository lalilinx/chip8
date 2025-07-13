use rand::Rng;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
// use winit::event::WindowEvent;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};
// use winit_input_helper::WinitInputHelper;

const FPS: f32 = 16.67; // 60 frames per second or 60 Hz
const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const FONTSET_START_ADDRESS: u16 = 0x50;

const INSTRUCTION_HZ: u64 = 700;
const RENDER_HZ: u64 = 120;
const TIMER_HZ: u64 = 60;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window({
                    let size =
                        LogicalSize::new(SCREEN_WIDTH as f64 * 10.0, SCREEN_HEIGHT as f64 * 10.0);
                    Window::default_attributes()
                        .with_title("Chip8 Emulator")
                        .with_inner_size(size)
                        .with_min_inner_size(size)
                })
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app);

    // let mut draw_flag: bool = false;
    // let instruction_interval = Duration::from_nanos(1_000_000_000 / INSTRUCTION_HZ);
    // let render_interval = Duration::from_nanos(1_000_000_000 / RENDER_HZ);
    // let timer_interval = Duration::from_nanos(1_000_000_000 / TIMER_HZ);

    // let mut last_instruction_tick = Instant::now();
    // let mut last_render_tick = Instant::now();
    // let mut last_timer_tick = Instant::now();

    // loop {
    //     let now = Instant::now();

    //     //using while loop for catch up missing execution instructions (gen-ai suggestion)
    //     while now.duration_since(last_instruction_tick) >= instruction_interval {
    //         // fetch, decode, and execute instructions
    //         last_instruction_tick += instruction_interval;
    //     }

    //     if draw_flag && now.duration_since(last_render_tick) >= render_interval {
    //         // render frame buffer to screen

    //         last_render_tick += render_interval;
    //         draw_flag = false;
    //     }

    //     while now.duration_since(last_timer_tick) >= timer_interval {
    //         // update delay_timers and sound_timers
    //         last_timer_tick += timer_interval;
    //     }

    //     let next_instruction_tick = last_instruction_tick + instruction_interval;
    //     let next_render_tick = last_render_tick + render_interval;
    //     let next_timer_tick = last_timer_tick + timer_interval;

    //     let next_tick = next_instruction_tick
    //         .min(next_render_tick)
    //         .min(next_timer_tick);
    //     let sleep_duration = next_tick.duration_since(now);
    //     if sleep_duration > Duration::from_millis(0) {
    //         std::thread::sleep(sleep_duration);
    //     }
    // }
}

struct Chip8 {
    registers: [u8; 16],
    i: u16,
    pc: u16,          // Program counter
    sp: u16,          // Stack pointer
    stack: [u16; 16], // Stack for storing return addresses
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],

    memory: [u8; 4096],

    draw_flag: bool,
    frame_buffer: [[u8; 63]; 32], // memory: Memory,
                                  // frame: [u8; ]
}

fn nibble(value: &u16, n: u8) -> u8 {
    ((value >> (n * 4)) & 0xF) as u8
}

impl Chip8 {
    fn init(&mut self) {
        self.pc = 0x200;
        self.sp = 0x0;

        //load font to memory
        //
    }
    fn cycle(&mut self) {
        // loop {
        let opcode: u16 = self.fetch();

        self.pc += 2;

        //decode and execute
        self.decode_and_execute(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    fn fetch(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 0x8
            | (self.memory[(self.pc + 0x1) as usize]) as u16
    }
    fn decode_and_execute(&mut self, inst: u16) {
        let _opcode: u16 = (inst >> 0xC) & 0xF;
        match _opcode {
            0x0 => {
                match inst & 0x0FFF {
                    0x00E0 => {
                        // clear screen
                        self.frame_buffer = [[0; 63]; 32];
                        self.draw_flag = true;
                    }
                    0x00EE => {
                        // return from subroutine
                        self.pc = match self.pop() {
                            Some(addr) => addr,
                            None => {
                                print!("Stack underflow");
                                return;
                            }
                        };
                    }
                    _ => (),
                }
            }
            0x1 => {
                let index: u16 = inst & 0x0FFF;
                self.pc = index;
            }
            0x2 => {
                let index: u16 = inst & 0x0FFF;
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
                let vx: u8 = self.get_register_data(&x);
                let kk: u8 = inst as u8;
                if vx == kk {
                    self.pc += 2;
                }
            }
            0x4 => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);
                let kk: u8 = inst as u8;
                if vx != kk {
                    self.pc += 2;
                }
            }
            0x5 => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);
                let y: u8 = nibble(&inst, 1);
                let vy: u8 = self.get_register_data(&y);
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
                let vx: u8 = self.get_register_data(&x);
                let kk: u8 = inst as u8;
                self.register(x, vx + kk);
            }
            0x8 => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);
                let y: u8 = nibble(&inst, 1);
                let vy: u8 = self.get_register_data(&y);

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
                let vx: u8 = self.get_register_data(&x);
                let y: u8 = nibble(&inst, 1);
                let vy: u8 = self.get_register_data(&y);

                if vx != vy {
                    self.pc -= 2;
                }
            }
            0xA => {
                let n: u16 = inst & 0x0FFF;
                self.i = n;
            }
            0xB => {
                let n: u16 = inst << 4;
                let v0: u8 = self.get_register_data(&0x0);
                self.pc = n + v0 as u16;
            }
            0xC => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);
                let kk: u8 = inst as u8;
                let ranNum: u8 = rand::thread_rng().r#gen();

                self.register(x, vx & kk);
            }
            0xD => {
                let x: u8 = nibble(&inst, 2);
                let vx = self.get_register_data(&x) as usize;
                let y: u8 = nibble(&inst, 1);
                let vy = self.get_register_data(&y) as usize;

                let n = (inst & 0x000F) as usize;
                let i = self.i as usize;

                let mut vf: u8 = 0x0;
                for row in 0..n {
                    let cur_sprite = self.memory[i + row];
                    for col in 0..8usize {
                        if cur_sprite & (0x80 >> col) == 1 {
                            let idx_x = (vx + col) % SCREEN_WIDTH as usize;
                            let idx_y = (vy + row) % SCREEN_HEIGHT as usize;
                            if self.frame_buffer[idx_y][idx_x] == 0x1 {
                                vf = 0x1;
                            }
                            self.frame_buffer[idx_y][idx_x] ^= 1;
                        }
                    }
                }
                self.register(0xF, vf);
            }
            0xE => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);

                let indic: u16 = inst & 0x00FF;
                match indic {
                    0x9E => {
                        let is_press = self.keypad[vx as usize];
                        if is_press {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        let is_press = self.keypad[vx as usize];
                        if !is_press {
                            self.pc += 2;
                        }
                    }
                    _ => (),
                }
            }
            0xF => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);

                let indic: u16 = inst & 0x00FF;
                match indic {
                    0x7 => {
                        self.register(x, self.delay_timer);
                    }
                    0xA => {
                        for key in 0..16 {
                            if self.keypad[key] {
                                self.register(x, key as u8);
                                return;
                            }
                        }
                        self.pc -= 2;
                    }
                    0x15 => {
                        self.delay_timer = vx;
                    }
                    0x18 => {
                        self.sound_timer = vx;
                    }
                    0x1E => {
                        self.i += vx as u16;
                    }
                    0x29 => {
                        self.i = FONTSET_START_ADDRESS + (vx as u16 * 5);
                    }
                    0x33 => {
                        self.memory[self.i as usize] = (vx / 100) as u8; // 100
                        self.memory[(self.i + 1) as usize] = ((vx / 10) % 10) as u8; // 10
                        self.memory[(self.i + 2) as usize] = (vx % 10) as u8; // 1
                    }
                    0x55 => {
                        for v in self.registers.iter().take((x + 1) as usize) {
                            self.memory[self.i as usize] = *v;
                            self.i += 1;
                        }
                    }
                    0x65 => {
                        for v in 0..x + 1 {
                            self.register(v, self.memory[self.i as usize]);
                            self.i += 1;
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn get_sprite(&self, i: &u16, n: &u8) {}

    fn get_register_data(&self, regi: &u8) -> u8 {
        self.registers[*regi as usize]
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
