mod gb;

use clap::Parser;
use gb::debug::{DebugConfig, MemoryRange, format_memory_dump};
use gb::debugger;
use gb::gameboy::*;
use gb::ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};
use gb::joypad::Button;
use minifb::{Key, Window, WindowOptions, Scale};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Game Boy emulator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the ROM file to load
    #[arg(short, long, value_name = "FILE")]
    file_path: PathBuf,

    /// Enable debug output (show CPU state after each instruction)
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// Use verbose multi-line debug format
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Maximum number of instructions to execute (for testing)
    #[arg(short, long, value_name = "N")]
    max_instructions: Option<u64>,

    /// Dump memory range after execution (format: START:END, e.g., 0x0000:0x00FF)
    #[arg(long = "dump-mem", value_name = "RANGE")]
    dump_memory: Vec<String>,

    /// Start interactive debugger
    #[arg(short, long, default_value_t = false)]
    interactive: bool,

    /// Run with display window (graphics mode)
    #[arg(long, default_value_t = false)]
    display: bool,

    /// Display scale factor (1-4)
    #[arg(long, default_value_t = 3)]
    scale: u8,
}

fn main() {
    let args = Args::parse();
    let file_name = args.file_path;
    let file_buf = load_file(file_name);

    let debug_config = DebugConfig::default()
        .with_debug(args.debug)
        .with_verbose(args.verbose)
        .with_max_instructions(args.max_instructions);

    let mut gb = GameBoy::new(file_buf, debug_config);

    if args.interactive {
        // Run TUI debugger
        if let Err(e) = debugger::run(&mut gb) {
            eprintln!("Debugger error: {}", e);
        }
    } else if args.display {
        // Run with display window
        if let Err(e) = run_with_display(&mut gb, args.scale) {
            eprintln!("Display error: {}", e);
        }
    } else {
        // Normal execution (headless)
        gb.run();

        // Memory dumps after execution
        for range_str in &args.dump_memory {
            match range_str.parse::<MemoryRange>() {
                Ok(range) => {
                    println!("\n--- Memory Dump 0x{:04X}:0x{:04X} ---", range.start, range.end);
                    print!("{}", format_memory_dump(gb.interconnect(), range));
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}

/// Run the emulator with a display window
fn run_with_display(gb: &mut GameBoy, scale: u8) -> Result<(), Box<dyn std::error::Error>> {
    let scale = match scale {
        1 => Scale::X1,
        2 => Scale::X2,
        4 => Scale::X4,
        _ => Scale::X2, // Default to 2x for scale=3 or invalid values
    };

    let mut window = Window::new(
        "Game Boy Emulator",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            scale,
            resize: false,
            ..WindowOptions::default()
        },
    )?;

    // Limit update rate to reduce CPU and ensure key events are processed
    window.set_target_fps(60);

    // Target ~60 FPS (16.67ms per frame)
    let frame_duration = Duration::from_micros(16742);

    // Game Boy runs at ~4.19 MHz, with 70224 cycles per frame
    const CYCLES_PER_FRAME: u32 = 70224;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();

        // Execute enough cycles for one frame
        let mut cycles_this_frame: u32 = 0;
        while cycles_this_frame < CYCLES_PER_FRAME {
            let cycles = gb.step();
            cycles_this_frame += cycles as u32;
        }

        // Update display (this also processes input events)
        if gb.interconnect().frame_ready() {
            let fb = gb.interconnect().framebuffer();
            window.update_with_buffer(fb, SCREEN_WIDTH, SCREEN_HEIGHT)?;
            gb.interconnect_mut().clear_frame_ready();
        } else {
            window.update();
        }

        // Handle input AFTER window.update() processes events
        handle_input(&window, gb);

        // Frame timing
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    Ok(())
}

/// Handle keyboard input and update joypad state
fn handle_input(window: &Window, gb: &mut GameBoy) {
    // Use get_keys() to get all currently pressed keys
    let keys = window.get_keys();
    let inter = gb.interconnect_mut();

    // D-pad
    if keys.contains(&Key::Right) {
        inter.press_button(Button::Right);
    } else {
        inter.release_button(Button::Right);
    }

    if keys.contains(&Key::Left) {
        inter.press_button(Button::Left);
    } else {
        inter.release_button(Button::Left);
    }

    if keys.contains(&Key::Up) {
        inter.press_button(Button::Up);
    } else {
        inter.release_button(Button::Up);
    }

    if keys.contains(&Key::Down) {
        inter.press_button(Button::Down);
    } else {
        inter.release_button(Button::Down);
    }

    // Action buttons (Z=A, X=B, Enter=Start, Backspace=Select)
    if keys.contains(&Key::Z) {
        inter.press_button(Button::A);
    } else {
        inter.release_button(Button::A);
    }

    if keys.contains(&Key::X) {
        inter.press_button(Button::B);
    } else {
        inter.release_button(Button::B);
    }

    if keys.contains(&Key::Enter) {
        inter.press_button(Button::Start);
    } else {
        inter.release_button(Button::Start);
    }

    if keys.contains(&Key::Backspace) {
        inter.press_button(Button::Select);
    } else {
        inter.release_button(Button::Select);
    }
}

fn load_file(file_name: PathBuf) -> Vec<u8> {
    let mut file = File::open(file_name).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}
