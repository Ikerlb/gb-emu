use crate::gb::debug::{format_memory_dump, parse_address, MemoryRange};
use crate::gb::gameboy::GameBoy;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Config, Context, EditMode, Editor, Helper};
use std::borrow::Cow;
use std::collections::HashSet;

// Minimalist ANSI codes - bold/dim for visual hierarchy
mod style {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
}

/// Debugger commands
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Step,
    Continue,
    Break(u16),
    Delete(usize),
    List,
    Registers,
    Memory(MemoryRange),
    Help,
    Quit,
}

/// Result of parsing a command
#[derive(Debug)]
pub enum ParseResult {
    Command(Command),
    Empty,
    Error(String),
}

/// Breakpoint manager
#[derive(Debug, Default)]
pub struct BreakpointManager {
    breakpoints: Vec<u16>,
    enabled: HashSet<usize>,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            enabled: HashSet::new(),
        }
    }

    /// Add a breakpoint at the given address, returns the breakpoint ID
    pub fn add(&mut self, addr: u16) -> usize {
        let id = self.breakpoints.len();
        self.breakpoints.push(addr);
        self.enabled.insert(id);
        id
    }

    /// Remove a breakpoint by ID
    pub fn remove(&mut self, id: usize) -> Result<u16, String> {
        if id >= self.breakpoints.len() {
            return Err(format!("Breakpoint {} does not exist", id));
        }
        self.enabled.remove(&id);
        Ok(self.breakpoints[id])
    }

    /// Check if we should break at the given address
    pub fn should_break(&self, addr: u16) -> bool {
        self.breakpoints
            .iter()
            .enumerate()
            .any(|(id, &bp_addr)| self.enabled.contains(&id) && bp_addr == addr)
    }

    /// Get all breakpoints for display
    pub fn list(&self) -> Vec<(usize, u16, bool)> {
        self.breakpoints
            .iter()
            .enumerate()
            .map(|(id, &addr)| (id, addr, self.enabled.contains(&id)))
            .collect()
    }

    /// Check if there are any enabled breakpoints
    pub fn has_enabled(&self) -> bool {
        !self.enabled.is_empty()
    }
}

/// Command hints for rustyline
#[derive(Debug)]
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: usize) -> Self {
        Self {
            display: text.to_string(),
            complete_up_to,
        }
    }
}

/// Available commands with their syntax hints
const COMMAND_HINTS: &[(&str, &str)] = &[
    ("step", ""),
    ("s", ""),
    ("continue", ""),
    ("c", ""),
    ("break", " <address>"),
    ("b", " <address>"),
    ("delete", " <id>"),
    ("d", " <id>"),
    ("list", ""),
    ("l", ""),
    ("reg", ""),
    ("r", ""),
    ("mem", " <start>:<end>"),
    ("m", " <start>:<end>"),
    ("help", ""),
    ("h", ""),
    ("quit", ""),
    ("q", ""),
];

/// Helper for rustyline with completion and hints
#[derive(Debug)]
struct DebuggerHelper {
    commands: Vec<String>,
}

impl DebuggerHelper {
    fn new() -> Self {
        Self {
            commands: COMMAND_HINTS.iter().map(|(cmd, _)| cmd.to_string()).collect(),
        }
    }
}

impl Completer for DebuggerHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let line_up_to_cursor = &line[..pos];
        let word_start = line_up_to_cursor.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let word = &line_up_to_cursor[word_start..];

        // Only complete command names (first word)
        if word_start == 0 {
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            Ok((word_start, matches))
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for DebuggerHelper {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if pos < line.len() {
            return None;
        }

        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        // Find matching command and show its syntax hint
        for (cmd, hint) in COMMAND_HINTS {
            if cmd.starts_with(line) && *cmd != line {
                let remaining = &cmd[line.len()..];
                return Some(CommandHint::new(
                    &format!("{}{}", remaining, hint),
                    remaining.len(),
                ));
            } else if *cmd == line && !hint.is_empty() {
                return Some(CommandHint::new(hint, 0));
            }
        }
        None
    }
}

impl Highlighter for DebuggerHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("{}{}{}", style::DIM, hint, style::RESET))
    }
}

impl Validator for DebuggerHelper {}
impl Helper for DebuggerHelper {}

/// Parse a command string into a Command
fn parse_command(input: &str) -> ParseResult {
    let input = input.trim();
    if input.is_empty() {
        return ParseResult::Empty;
    }

    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();

    match cmd {
        "step" | "s" => ParseResult::Command(Command::Step),
        "continue" | "c" => ParseResult::Command(Command::Continue),
        "break" | "b" => {
            if args.is_empty() {
                return ParseResult::Error("Usage: break <address>".to_string());
            }
            match parse_address(args[0]) {
                Ok(addr) => ParseResult::Command(Command::Break(addr)),
                Err(e) => ParseResult::Error(e),
            }
        }
        "delete" | "d" => {
            if args.is_empty() {
                return ParseResult::Error("Usage: delete <breakpoint_id>".to_string());
            }
            match args[0].parse::<usize>() {
                Ok(id) => ParseResult::Command(Command::Delete(id)),
                Err(_) => ParseResult::Error(format!("Invalid breakpoint ID: {}", args[0])),
            }
        }
        "list" | "l" => ParseResult::Command(Command::List),
        "reg" | "r" => ParseResult::Command(Command::Registers),
        "mem" | "m" => {
            if args.is_empty() {
                return ParseResult::Error("Usage: mem <start>:<end>".to_string());
            }
            match args[0].parse::<MemoryRange>() {
                Ok(range) => ParseResult::Command(Command::Memory(range)),
                Err(e) => ParseResult::Error(e),
            }
        }
        "help" | "h" => ParseResult::Command(Command::Help),
        "quit" | "q" => ParseResult::Command(Command::Quit),
        _ => ParseResult::Error(format!("Unknown command: {}. Type 'help' for commands.", cmd)),
    }
}

/// Interactive debugger
pub struct Debugger {
    editor: Editor<DebuggerHelper, DefaultHistory>,
    breakpoints: BreakpointManager,
    last_command: Option<Command>,
    history_path: Option<std::path::PathBuf>,
}

impl Debugger {
    pub fn new() -> Result<Self, String> {
        let config = Config::builder()
            .edit_mode(EditMode::Vi)
            .auto_add_history(true)
            .build();

        let mut editor = Editor::with_config(config)
            .map_err(|e| format!("Failed to create editor: {}", e))?;
        editor.set_helper(Some(DebuggerHelper::new()));

        // Try to load history
        let history_path = dirs::home_dir().map(|p| p.join(".gb_emu_history"));
        if let Some(ref path) = history_path {
            let _ = editor.load_history(path);
        }

        Ok(Self {
            editor,
            breakpoints: BreakpointManager::new(),
            last_command: None,
            history_path,
        })
    }

    /// Run the interactive debugger
    pub fn run(&mut self, gb: &mut GameBoy) {
        println!("{}─ GAME BOY DEBUGGER ─{}", style::BOLD, style::RESET);
        println!("{}Type 'help' for commands, 'quit' to exit{}", style::DIM, style::RESET);
        println!("{}Vi mode: ESC for normal, i for insert{}\n", style::DIM, style::RESET);

        // Show initial state
        self.print_cpu_state(gb);

        loop {
            let prompt = format!("({:04X}) λ ", gb.cpu().pc());
            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let result = parse_command(&line);
                    match result {
                        ParseResult::Command(cmd) => {
                            self.last_command = Some(cmd.clone());
                            if !self.execute_command(cmd, gb) {
                                break;
                            }
                        }
                        ParseResult::Empty => {
                            // Repeat last command on empty input
                            if let Some(cmd) = self.last_command.clone() {
                                if !self.execute_command(cmd, gb) {
                                    break;
                                }
                            }
                        }
                        ParseResult::Error(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Ctrl+C pressed. Type 'quit' to exit.");
                }
                Err(ReadlineError::Eof) => {
                    println!("Ctrl+D pressed, exiting.");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        if let Some(ref path) = self.history_path {
            let _ = self.editor.save_history(path);
        }
    }

    /// Execute a command, returns false if debugger should exit
    fn execute_command(&mut self, cmd: Command, gb: &mut GameBoy) -> bool {
        match cmd {
            Command::Step => {
                gb.step();
                self.print_cpu_state(gb);
            }
            Command::Continue => {
                self.run_until_breakpoint(gb);
            }
            Command::Break(addr) => {
                let id = self.breakpoints.add(addr);
                println!("Breakpoint {} set at 0x{:04X}", id, addr);
            }
            Command::Delete(id) => match self.breakpoints.remove(id) {
                Ok(addr) => println!("Deleted breakpoint {} (was at 0x{:04X})", id, addr),
                Err(e) => eprintln!("Error: {}", e),
            },
            Command::List => {
                let bps = self.breakpoints.list();
                if bps.is_empty() {
                    println!("No breakpoints set.");
                } else {
                    println!("Breakpoints:");
                    for (id, addr, enabled) in bps {
                        let status = if enabled { "" } else { " (disabled)" };
                        println!("  {}: 0x{:04X}{}", id, addr, status);
                    }
                }
            }
            Command::Registers => {
                self.print_cpu_state(gb);
            }
            Command::Memory(range) => {
                println!(
                    "Memory 0x{:04X}:0x{:04X}",
                    range.start, range.end
                );
                print!("{}", format_memory_dump(gb.interconnect(), range));
            }
            Command::Help => {
                self.print_help();
            }
            Command::Quit => {
                return false;
            }
        }
        true
    }

    /// Run until a breakpoint is hit or execution halts
    fn run_until_breakpoint(&mut self, gb: &mut GameBoy) {
        if !self.breakpoints.has_enabled() {
            println!("No breakpoints set. Use 'step' or set a breakpoint first.");
            return;
        }

        let mut steps = 0u64;
        loop {
            let pc = gb.cpu().pc();
            if self.breakpoints.should_break(pc) && steps > 0 {
                println!("{}Breakpoint{} at {:04X}", style::BOLD, style::RESET, pc);
                self.print_cpu_state(gb);
                break;
            }

            let halted = gb.step();
            steps += 1;

            if halted {
                println!("{}Halted{} after {} instructions", style::DIM, style::RESET, steps);
                self.print_cpu_state(gb);
                break;
            }

            // Safety limit to prevent infinite loops
            if steps > 10_000_000 {
                println!("{}Stopped{} after {} instructions", style::DIM, style::RESET, steps);
                self.print_cpu_state(gb);
                break;
            }
        }
    }

    fn print_help(&self) {
        println!(
            r#"Commands:
  step, s           Execute one instruction
  continue, c       Run until breakpoint or halt
  break, b <addr>   Set breakpoint at address (e.g., b 0x150)
  delete, d <id>    Delete breakpoint by ID
  list, l           List all breakpoints
  reg, r            Show CPU registers
  mem, m <range>    Dump memory (e.g., m 0x0000:0x00FF)
  help, h           Show this help
  quit, q           Exit debugger

Tips:
  - Press Enter with empty input to repeat last command
  - Vi keybindings: ESC for normal mode, i for insert
  - Tab to autocomplete commands
  - Up/Down arrows for command history"#
        );
    }

    /// Print CPU state with visual hierarchy (dim labels, bold values)
    fn print_cpu_state(&self, gb: &GameBoy) {
        let cpu = gb.cpu();

        // Registers line: dim labels, bold values
        let regs = [
            ("PC", cpu.pc()),
            ("SP", cpu.sp()),
            ("AF", cpu.af()),
            ("BC", cpu.bc()),
            ("DE", cpu.de()),
            ("HL", cpu.hl()),
        ];
        let reg_line: Vec<String> = regs
            .iter()
            .map(|(name, val)| {
                format!(
                    "{}{}:{}{}{:04X}{}",
                    style::DIM, name, style::RESET, style::BOLD, val, style::RESET
                )
            })
            .collect();
        println!("{}", reg_line.join("  "));

        // Flags: uppercase=set, lowercase=unset
        let f = (cpu.af() & 0xFF) as u8;
        let flags = [('Z', 0x80), ('N', 0x40), ('H', 0x20), ('C', 0x10)];
        let flag_strs: Vec<String> = flags
            .iter()
            .map(|(name, mask)| {
                if f & mask != 0 {
                    format!("{}{}{}", style::BOLD, name, style::RESET)
                } else {
                    format!("{}{}{}", style::DIM, name.to_ascii_lowercase(), style::RESET)
                }
            })
            .collect();
        println!("{}Flags:{} [ {} ]", style::DIM, style::RESET, flag_strs.join(" "));
    }
}
