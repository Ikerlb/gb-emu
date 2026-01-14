use crate::gb::debug::{parse_address, MemoryRange};
use std::collections::HashSet;

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

/// Parse a command string into a Command
pub fn parse_command(input: &str) -> ParseResult {
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
