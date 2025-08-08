use crossbeam_channel::{select, unbounded};
use pixels::{Error, Pixels, SurfaceTexture};
use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const FPS: f32 = 16.67; // 60 frames per second or 60 Hz
const FPS60: Duration = Duration::from_micros(16_67);
// const FPS60: Duration = Duration::from_secs(3);
const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const FONTSET_START_ADDRESS: u16 = 0x50;

const MEMORY_SIZE: usize = 4096;
const PROGRAM_START_LOC: usize = 0x200;

const FONT_START_LOC: usize = 0x50;

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

fn main() -> Result<(), Error> {
    // let (tx, rx) = mpsc::channel::<&[u8]>();
    let (sender, reciever) = unbounded::<KeyEvent>();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64 * 10.0, SCREEN_HEIGHT as f64 * 10.0);
        WindowBuilder::new()
            .with_title("Chip8 Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let screen_buffer = Arc::new(Mutex::new([[1u8; 64]; 32]));
    let _screen_buffer = Arc::clone(&screen_buffer);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?
    };

    let worker = thread::spawn(move || {
        let mut chip8 = Chip8::new_with_buffer(_screen_buffer);
        chip8.init();

        let mut draw_flag: bool = false;
        let instruction_interval = Duration::from_nanos(1_000_000_000 / INSTRUCTION_HZ);
        let render_interval = Duration::from_nanos(1_000_000_000 / RENDER_HZ);
        let timer_interval = Duration::from_nanos(1_000_000_000 / TIMER_HZ);

        let mut last_instruction_tick = Instant::now();
        let mut last_render_tick = Instant::now();
        let mut last_timer_tick = Instant::now();

        // loop for chip8 emulator
        loop {
            let now = Instant::now();

            select! {
                recv(reciever) -> message => {
                    //update_input
                    match message {
                        Ok(result) => {
                            chip8.handle_input(result);
                            chip8.input_handler.display_key_states();
                        },
                        Err(error) => {
                            println!("Error: {}", error);
                            break
                        },
                    }

                },
                default() => {}
            }

            //using while loop for catch up missing execution instructions (gen-ai suggestion)
            while now.duration_since(last_instruction_tick) >= instruction_interval {
                // fetch, decode, and execute instructions
                //chip8 fetch
                //chip8 decode and execute
                last_instruction_tick += instruction_interval;
            }

            if draw_flag && now.duration_since(last_render_tick) >= render_interval {
                // render frame buffer to screen
                // let buffer = _screen_buffer.lock().unwrap();
                last_render_tick += render_interval;
                draw_flag = false;
            }

            while now.duration_since(last_timer_tick) >= timer_interval {
                // update delay_timers and sound_timers
                last_timer_tick += timer_interval;
            }

            let next_instruction_tick = last_instruction_tick + instruction_interval;
            let next_render_tick = last_render_tick + render_interval;
            let next_timer_tick = last_timer_tick + timer_interval;

            let next_tick = next_instruction_tick
                .min(next_render_tick)
                .min(next_timer_tick);
            let sleep_duration = next_tick.duration_since(now);
            if sleep_duration > Duration::from_millis(0) {
                std::thread::sleep(sleep_duration);
            }
        }
    });

    let res = event_loop.run(|event, event_loop_window_target| {
        // println!("Event: {:?}", event);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                event_loop_window_target.exit();
            }

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if let Err(send_err) = sender.try_send(event) {
                    use crossbeam_channel::TrySendError;
                    match send_err {
                        TrySendError::Full(f) => {}
                        TrySendError::Disconnected(d) => {
                            println!("Disconnected receiver")
                        }
                    }
                    event_loop_window_target.exit();
                }
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Redraw the window
                let buf = screen_buffer.lock().unwrap();
                let frame = pixels.frame_mut();
                for (i, row) in buf.iter().enumerate() {
                    for (j, col) in row.iter().enumerate() {
                        let v = if *col == 1 { 0xFF } else { 0x00 };
                        let ofset = (i * SCREEN_WIDTH as usize + j) * 4;
                        frame[ofset..ofset + 4].copy_from_slice(&[v, v, v, 0xFF]);
                    }
                }

                if pixels.render().is_err() {
                    event_loop_window_target.exit();
                }

                event_loop_window_target
                    .set_control_flow(ControlFlow::WaitUntil(Instant::now() + FPS60));
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });

    println!("shuting down...");
    //close sender
    drop(sender);
    worker.join().unwrap();
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

struct InputHandler {
    keypad: [bool; 16],
    key_mapping: HashMap<KeyCode, u8>,
}

impl InputHandler {
    fn new() -> Self {
        let mut key_mapping = HashMap::new();

        key_mapping.insert(KeyCode::Digit1, 0x1);
        key_mapping.insert(KeyCode::Digit2, 0x2);
        key_mapping.insert(KeyCode::Digit3, 0x3);
        key_mapping.insert(KeyCode::Digit4, 0xC);

        key_mapping.insert(KeyCode::KeyQ, 0x4);
        key_mapping.insert(KeyCode::KeyW, 0x5);
        key_mapping.insert(KeyCode::KeyE, 0x6);
        key_mapping.insert(KeyCode::KeyR, 0xD);

        key_mapping.insert(KeyCode::KeyA, 0x7);
        key_mapping.insert(KeyCode::KeyS, 0x8);
        key_mapping.insert(KeyCode::KeyD, 0x9);
        key_mapping.insert(KeyCode::KeyF, 0xE);

        key_mapping.insert(KeyCode::KeyZ, 0xA);
        key_mapping.insert(KeyCode::KeyX, 0x0);
        key_mapping.insert(KeyCode::KeyC, 0xB);
        key_mapping.insert(KeyCode::KeyV, 0xF);

        Self {
            keypad: [false; 16],
            key_mapping,
        }
    }

    fn key_pressed(&mut self, key_code: KeyCode) {
        if let Some(&chip8_key) = self.key_mapping.get(&key_code) {
            self.keypad[chip8_key as usize] = true;
        }
    }
    fn key_released(&mut self, key_code: KeyCode) {
        if let Some(&chip8_key) = self.key_mapping.get(&key_code) {
            self.keypad[chip8_key as usize] = false;
        }
    }
    fn is_key_pressed(&self, key: u8) -> bool {
        if key < 16 {
            self.keypad[key as usize]
        } else {
            false
        }
    }

    fn display_key_states(&self) {
        println!("CHIP-8 Key States:");
        println!(
            "1:{} 2:{} 3:{} C:{}",
            if self.keypad[0x1] { "█" } else { "." },
            if self.keypad[0x2] { "█" } else { "." },
            if self.keypad[0x3] { "█" } else { "." },
            if self.keypad[0xC] { "█" } else { "." }
        );
        println!(
            "4:{} 5:{} 6:{} D:{}",
            if self.keypad[0x4] { "█" } else { "." },
            if self.keypad[0x5] { "█" } else { "." },
            if self.keypad[0x6] { "█" } else { "." },
            if self.keypad[0xD] { "█" } else { "." }
        );
        println!(
            "7:{} 8:{} 9:{} E:{}",
            if self.keypad[0x7] { "█" } else { "." },
            if self.keypad[0x8] { "█" } else { "." },
            if self.keypad[0x9] { "█" } else { "." },
            if self.keypad[0xE] { "█" } else { "." }
        );
        println!(
            "A:{} 0:{} B:{} F:{}",
            if self.keypad[0xA] { "█" } else { "." },
            if self.keypad[0x0] { "█" } else { "." },
            if self.keypad[0xB] { "█" } else { "." },
            if self.keypad[0xF] { "█" } else { "." }
        );
        println!();
    }
}

struct Chip8 {
    registers: [u8; 16],
    i: u16,
    pc: u16,          // Program counter
    sp: u16,          // Stack pointer
    stack: [u16; 16], // Stack for storing return addresses
    delay_timer: u8,
    sound_timer: u8,
    // keypad: [bool; 16],
    memory: [u8; 4096],

    input_handler: InputHandler,

    draw_flag: bool,
    frame_buffer: Arc<Mutex<[[u8; 64]; 32]>>, // memory: Memory,
                                              // frame: [u8; ]
}

fn nibble(value: &u16, n: u8) -> u8 {
    ((value >> (n * 4)) & 0xF) as u8
}

impl Chip8 {
    fn new_with_buffer(buffer: Arc<Mutex<[[u8; 64]; 32]>>) -> Self {
        Self {
            registers: [0x0; 16],
            i: 0x0,
            pc: 0x0,
            sp: 0x0,
            stack: [0x0; 16],
            delay_timer: 0x0,
            sound_timer: 0x0,
            // keypad: [false; 16],
            input_handler: InputHandler::new(),
            memory: [0x0; 4096],
            draw_flag: false,
            frame_buffer: buffer,
        }
    }

    fn init(&mut self) {
        self.pc = PROGRAM_START_LOC as u16;
        self.sp = 0x0;

        //load font to memory
        for (i, &font) in FONTSET.iter().enumerate() {
            self.memory[FONT_START_LOC + i] = font;
        }
    }

    fn load_rom(&mut self, path: String) -> Result<(), io::Error> {
        let rom_data = fs::read(path)?;

        if rom_data.len() > MEMORY_SIZE - PROGRAM_START_LOC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ROM file too large",
            ));
        }

        for (i, &byte) in rom_data.iter().enumerate() {
            self.memory[PROGRAM_START_LOC + i] = byte;
        }
        println!("Loaded ROM: {} bytes", rom_data.len());
        Ok(())
    }

    fn handle_input(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.input_handler.key_pressed(key_code);
                }
                ElementState::Released => {
                    self.input_handler.key_released(key_code);
                }
            }
        }
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
                        {
                            *self.frame_buffer.lock().unwrap() = [[0; 64]; 32];
                            self.draw_flag = true;
                        }
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
                {
                    let mut buffer = self.frame_buffer.lock().unwrap();
                    for row in 0..n {
                        let cur_sprite = self.memory[i + row];
                        for col in 0..8usize {
                            if cur_sprite & (0x80 >> col) != 0 {
                                let idx_x = (vx + col) % SCREEN_WIDTH as usize;
                                let idx_y = (vy + row) % SCREEN_HEIGHT as usize;
                                if buffer[idx_y][idx_x] == 0x1 {
                                    vf = 0x1;
                                }
                                buffer[idx_y][idx_x] ^= 1;
                            }
                        }
                    }
                }
                self.register(0xF, vf);
            }
            0xE => {
                let x: u8 = nibble(&inst, 2);
                let vx: u8 = self.get_register_data(&x);

                let indic: u16 = inst & 0x00FF;

                let is_press = self.input_handler.is_key_pressed(vx);
                match indic {
                    0x9E => {
                        // let is_press = self.keypad[vx as usize];
                        if is_press {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // let is_press = self.keypad[vx as usize];
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
                            if self.input_handler.is_key_pressed(key) {
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
