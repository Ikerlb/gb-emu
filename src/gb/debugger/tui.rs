use crate::gb::debug::MemoryRange;
use crate::gb::gameboy::GameBoy;

use super::core::{parse_command, BreakpointManager, Command, ParseResult};

use crossterm::{
    event::{self, Event, MouseEvent, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::io::stdout;
use tui_textarea::{Input, Key, TextArea};

/// Available bytes per line options (must be sorted ascending)
const BYTES_PER_LINE_OPTIONS: &[u16] = &[4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32];

/// Which panel has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Input,
    Memory,
}

/// Register selected for modal display
#[derive(Debug, Clone, Copy)]
enum SelectedRegister {
    PC,
    SP,
    AF,
    BC,
    DE,
    HL,
}

impl SelectedRegister {
    /// Returns (name, hi_name, lo_name)
    fn names(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            SelectedRegister::PC => ("PC", "P", "C"),
            SelectedRegister::SP => ("SP", "S", "P"),
            SelectedRegister::AF => ("AF", "A", "F"),
            SelectedRegister::BC => ("BC", "B", "C"),
            SelectedRegister::DE => ("DE", "D", "E"),
            SelectedRegister::HL => ("HL", "H", "L"),
        }
    }

    fn value(&self, gb: &GameBoy) -> u16 {
        match self {
            SelectedRegister::PC => gb.cpu().pc(),
            SelectedRegister::SP => gb.cpu().sp(),
            SelectedRegister::AF => gb.cpu().af(),
            SelectedRegister::BC => gb.cpu().bc(),
            SelectedRegister::DE => gb.cpu().de(),
            SelectedRegister::HL => gb.cpu().hl(),
        }
    }
}

/// TUI Debugger state
pub struct TuiDebugger<'a> {
    breakpoints: BreakpointManager,
    textarea: TextArea<'a>,
    last_command: Option<Command>,
    /// Start address for memory viewer (aligned to bytes per line)
    memory_offset: u16,
    /// Number of lines visible in memory viewer
    memory_lines: u16,
    /// Bytes per line (responsive to terminal width)
    bytes_per_line: u16,
    /// Status message shown in command input title
    status: String,
    /// Command history
    history: Vec<String>,
    /// Current position in history (0 = current input, 1 = last command, etc.)
    history_index: usize,
    /// Saved current input when browsing history
    saved_input: String,
    /// Which panel has focus
    focus: Focus,
    /// Highlighted memory range (from mem command)
    highlight: Option<MemoryRange>,
    /// Area where registers panel is rendered (for click detection)
    registers_area: Rect,
    /// Currently open register modal (if any)
    register_modal: Option<SelectedRegister>,
}

impl<'a> TuiDebugger<'a> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());

        Self {
            breakpoints: BreakpointManager::new(),
            textarea,
            last_command: None,
            memory_offset: 0x0100, // Start at typical ROM entry point
            memory_lines: 10,
            bytes_per_line: 16,
            status: "Tab=switch focus, scroll=memory".to_string(),
            history: Vec::new(),
            history_index: 0,
            saved_input: String::new(),
            focus: Focus::Input,
            highlight: None,
            registers_area: Rect::default(),
            register_modal: None,
        }
    }

    /// Add command to history
    fn add_to_history(&mut self, cmd: &str) {
        let cmd = cmd.trim().to_string();
        if !cmd.is_empty() {
            // Don't add duplicates of the last command
            if self.history.last() != Some(&cmd) {
                self.history.push(cmd);
            }
        }
        self.history_index = 0;
        self.saved_input.clear();
    }

    /// Navigate history up (older commands)
    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Save current input when first going into history
        if self.history_index == 0 {
            self.saved_input = self.textarea.lines()[0].clone();
        }

        if self.history_index < self.history.len() {
            self.history_index += 1;
            let cmd = self.history[self.history.len() - self.history_index].clone();
            self.textarea.select_all();
            self.textarea.delete_char();
            self.textarea.insert_str(&cmd);
        }
    }

    /// Navigate history down (newer commands)
    fn history_down(&mut self) {
        if self.history_index == 0 {
            return;
        }

        self.history_index -= 1;

        let text = if self.history_index == 0 {
            self.saved_input.clone()
        } else {
            self.history[self.history.len() - self.history_index].clone()
        };

        self.textarea.select_all();
        self.textarea.delete_char();
        self.textarea.insert_str(&text);
    }

    fn set_status(&mut self, msg: String) {
        self.status = msg;
    }

    /// Scroll memory view by a number of lines (positive = down, negative = up)
    fn scroll_memory(&mut self, lines: i32) {
        let delta = (lines as i32) * (self.bytes_per_line as i32);
        let new_offset = (self.memory_offset as i32).saturating_add(delta);
        self.memory_offset = new_offset.clamp(0, 0xFFFF) as u16;
        // Align to bytes_per_line boundary
        let mask = !(self.bytes_per_line - 1);
        self.memory_offset &= mask;
        // Clear highlight when scrolling manually
        self.highlight = None;
    }

    /// Jump memory view to contain a specific range and highlight it
    fn goto_memory(&mut self, range: MemoryRange) {
        // Align start to bytes_per_line boundary
        let mask = !(self.bytes_per_line - 1);
        self.memory_offset = range.start & mask;
        // Set highlight
        self.highlight = Some(range);
    }

    /// Follow PC - scroll memory view to show current program counter
    fn follow_pc(&mut self, gb: &GameBoy) {
        let pc = gb.cpu().pc();
        let mask = !(self.bytes_per_line - 1);
        self.memory_offset = pc & mask;
        self.highlight = None;
    }

    fn execute_command(&mut self, cmd: Command, gb: &mut GameBoy) -> bool {
        match cmd {
            Command::Step => {
                gb.step();
                self.follow_pc(gb);
                self.set_status(format!("Stepped to 0x{:04X}", gb.cpu().pc()));
            }
            Command::Continue => {
                if !self.breakpoints.has_enabled() {
                    self.set_status("No breakpoints set. Use 'break <addr>' first.".to_string());
                } else {
                    self.run_until_breakpoint(gb);
                }
            }
            Command::Break(addr) => {
                let id = self.breakpoints.add(addr);
                self.set_status(format!("Breakpoint {} set at 0x{:04X}", id, addr));
            }
            Command::Delete(id) => match self.breakpoints.remove(id) {
                Ok(addr) => self.set_status(format!("Deleted breakpoint {} (was 0x{:04X})", id, addr)),
                Err(e) => self.set_status(format!("Error: {}", e)),
            },
            Command::List => {
                let bps = self.breakpoints.list();
                if bps.is_empty() {
                    self.set_status("No breakpoints set.".to_string());
                } else {
                    let list: Vec<String> = bps
                        .iter()
                        .filter(|(_, _, enabled)| *enabled)
                        .map(|(id, addr, _)| format!("{}:0x{:04X}", id, addr))
                        .collect();
                    self.set_status(format!("Breakpoints: {}", list.join(", ")));
                }
            }
            Command::Registers => {
                self.set_status("(see Registers panel)".to_string());
            }
            Command::Memory(range) => {
                let start = range.start;
                let end = range.end;
                self.goto_memory(range);
                self.set_status(format!("Highlighted 0x{:04X}-0x{:04X}", start, end));
            }
            Command::Help => {
                self.set_status("s=step c=cont b=break d=del l=list m=mem q=quit".to_string());
            }
            Command::Quit => {
                return false;
            }
        }
        true
    }

    fn run_until_breakpoint(&mut self, gb: &mut GameBoy) {
        let mut steps = 0u64;
        loop {
            let pc = gb.cpu().pc();
            if self.breakpoints.should_break(pc) && steps > 0 {
                self.set_status(format!("Breakpoint hit at 0x{:04X} ({} steps)", pc, steps));
                break;
            }

            let halted = gb.step();
            steps += 1;

            if halted {
                self.set_status(format!("Halted after {} instructions", steps));
                break;
            }

            if steps > 10_000_000 {
                self.set_status(format!("Stopped after {} instructions (limit)", steps));
                break;
            }
        }
        self.follow_pc(gb);
    }

    fn handle_input(&mut self, input: Input, gb: &mut GameBoy) -> bool {
        match input {
            Input { key: Key::Tab, .. } => {
                // Switch focus between panels
                self.focus = match self.focus {
                    Focus::Input => Focus::Memory,
                    Focus::Memory => Focus::Input,
                };
                self.update_status_for_focus();
            }
            Input { key: Key::Up, .. } => {
                match self.focus {
                    Focus::Input => self.history_up(),
                    Focus::Memory => self.scroll_memory(-1),
                }
            }
            Input { key: Key::Down, .. } => {
                match self.focus {
                    Focus::Input => self.history_down(),
                    Focus::Memory => self.scroll_memory(1),
                }
            }
            Input { key: Key::Enter, .. } => {
                // Enter always executes command, regardless of focus
                self.focus = Focus::Input;
                let line = self.textarea.lines()[0].clone();
                self.add_to_history(&line);
                self.textarea.select_all();
                self.textarea.delete_char();

                let result = parse_command(&line);
                match result {
                    ParseResult::Command(cmd) => {
                        self.last_command = Some(cmd.clone());
                        if !self.execute_command(cmd, gb) {
                            return false;
                        }
                    }
                    ParseResult::Empty => {
                        if let Some(cmd) = self.last_command.clone() {
                            if !self.execute_command(cmd, gb) {
                                return false;
                            }
                        }
                    }
                    ParseResult::Error(e) => {
                        self.set_status(format!("Error: {}", e));
                    }
                }
            }
            Input { key: Key::Esc, .. } => {
                // Close modal if open, otherwise clear input
                if self.register_modal.is_some() {
                    self.register_modal = None;
                } else {
                    self.focus = Focus::Input;
                    self.history_index = 0;
                    self.textarea.select_all();
                    self.textarea.delete_char();
                    self.update_status_for_focus();
                }
            }
            Input {
                key: Key::Char('c'),
                ctrl: true,
                ..
            } => {
                return false;
            }
            _ => {
                // Other keys go to input (and switch focus to input)
                if self.focus == Focus::Memory {
                    // If typing while in memory focus, switch to input
                    if let Key::Char(_) = input.key {
                        self.focus = Focus::Input;
                        self.update_status_for_focus();
                    }
                }
                self.textarea.input(input);
            }
        }
        true
    }

    fn handle_mouse(&mut self, event: MouseEvent, gb: &GameBoy) {
        match event.kind {
            MouseEventKind::ScrollUp => {
                if self.focus == Focus::Memory {
                    self.scroll_memory(-3);
                }
            }
            MouseEventKind::ScrollDown => {
                if self.focus == Focus::Memory {
                    self.scroll_memory(3);
                }
            }
            MouseEventKind::Down(_) => {
                // If modal is open, close it on any click
                if self.register_modal.is_some() {
                    self.register_modal = None;
                } else if let Some(reg) = self.get_clicked_register(event.column, event.row) {
                    self.register_modal = Some(reg);
                }
            }
            _ => {}
        }
    }

    /// Determine which register was clicked
    fn get_clicked_register(&self, col: u16, row: u16) -> Option<SelectedRegister> {
        let area = self.registers_area;

        // Check if click is within the registers panel (excluding border)
        if col <= area.x || col >= area.x + area.width - 1 {
            return None;
        }
        if row <= area.y || row >= area.y + area.height - 1 {
            return None;
        }

        // Convert to content-relative coordinates (subtract border)
        let content_col = col - area.x - 1;
        let content_row = row - area.y - 1;

        // Layout: "PC:0100  SP:FFFE" (col 0-6 = PC, col 9-15 = SP)
        //         "AF:01B0  BC:0013"
        //         "DE:00D8  HL:014D"
        match (content_row, content_col) {
            (0, 0..=6) => Some(SelectedRegister::PC),
            (0, 9..=15) => Some(SelectedRegister::SP),
            (1, 0..=6) => Some(SelectedRegister::AF),
            (1, 9..=15) => Some(SelectedRegister::BC),
            (2, 0..=6) => Some(SelectedRegister::DE),
            (2, 9..=15) => Some(SelectedRegister::HL),
            _ => None,
        }
    }

    fn update_status_for_focus(&mut self) {
        match self.focus {
            Focus::Input => self.set_status("Tab=memory, Up/Down=history".to_string()),
            Focus::Memory => self.set_status("Tab=input, Up/Down/Scroll=navigate".to_string()),
        }
    }

    /// Update command input block with current status and focus
    fn update_input_block(&mut self) {
        let title = format!(" {} ", self.status);
        let style = if self.focus == Focus::Input {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style)
        );
    }
}

fn render_registers(cpu: &crate::gb::cpu::Cpu) -> Paragraph<'static> {
    let clickable = Style::default().fg(Color::Black).bg(Color::Gray).add_modifier(Modifier::UNDERLINED);
    let normal = Style::default();

    let lines = vec![
        Line::from(vec![
            Span::styled("PC:", normal),
            Span::styled(format!("{:04X}", cpu.pc()), clickable),
            Span::raw("  "),
            Span::styled("SP:", normal),
            Span::styled(format!("{:04X}", cpu.sp()), clickable),
        ]),
        Line::from(vec![
            Span::styled("AF:", normal),
            Span::styled(format!("{:04X}", cpu.af()), clickable),
            Span::raw("  "),
            Span::styled("BC:", normal),
            Span::styled(format!("{:04X}", cpu.bc()), clickable),
        ]),
        Line::from(vec![
            Span::styled("DE:", normal),
            Span::styled(format!("{:04X}", cpu.de()), clickable),
            Span::raw("  "),
            Span::styled("HL:", normal),
            Span::styled(format!("{:04X}", cpu.hl()), clickable),
        ]),
    ];

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Registers "))
}

fn render_flags(cpu: &crate::gb::cpu::Cpu) -> Paragraph<'static> {
    let f = (cpu.af() & 0xFF) as u8;
    let z = if f & 0x80 != 0 { "1" } else { "0" };
    let n = if f & 0x40 != 0 { "1" } else { "0" };
    let h = if f & 0x20 != 0 { "1" } else { "0" };
    let c = if f & 0x10 != 0 { "1" } else { "0" };

    let text = format!("Z:{}  N:{}\nH:{}  C:{}", z, n, h, c);
    Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Flags "))
}

fn render_memory(
    gb: &GameBoy,
    offset: u16,
    lines: u16,
    bytes_per_line: u16,
    focused: bool,
    highlight: &Option<MemoryRange>,
) -> Paragraph<'static> {
    let end_addr = offset.saturating_add(lines * bytes_per_line - 1).min(0xFFFF);
    let title = format!(" Memory 0x{:04X}-0x{:04X} ", offset, end_addr);

    let highlight_style = Style::default().fg(Color::Black).bg(Color::Yellow);
    let normal_style = Style::default();

    let mut text_lines: Vec<Line> = Vec::new();

    for line_idx in 0..lines {
        let line_addr = offset.saturating_add(line_idx * bytes_per_line);

        let mut spans: Vec<Span> = Vec::new();

        // Address
        spans.push(Span::styled(format!("{:04X} │ ", line_addr), normal_style));

        // Hex bytes
        for i in 0..bytes_per_line {
            let addr = line_addr.saturating_add(i);
            let byte_str = format!("{:02X}", gb.interconnect().read(addr));

            let is_highlighted = highlight
                .as_ref()
                .map(|r| r.contains(addr))
                .unwrap_or(false);

            let style = if is_highlighted { highlight_style } else { normal_style };
            spans.push(Span::styled(byte_str, style));

            // Space after each byte, extra space at midpoint
            if i == bytes_per_line / 2 - 1 {
                spans.push(Span::raw("  "));
            } else {
                spans.push(Span::raw(" "));
            }
        }

        // Separator
        spans.push(Span::styled("│ ", normal_style));

        // ASCII representation
        for i in 0..bytes_per_line {
            let addr = line_addr.saturating_add(i);
            let b = gb.interconnect().read(addr);
            let ascii_char = if (0x20..=0x7E).contains(&b) { b as char } else { '.' };

            let is_highlighted = highlight
                .as_ref()
                .map(|r| r.contains(addr))
                .unwrap_or(false);

            let style = if is_highlighted { highlight_style } else { normal_style };
            spans.push(Span::styled(ascii_char.to_string(), style));
        }

        text_lines.push(Line::from(spans));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    Paragraph::new(text_lines).block(block)
}

fn render_breakpoints(breakpoints: &BreakpointManager) -> Paragraph<'static> {
    let bps = breakpoints.list();
    let text = if bps.is_empty() {
        "(none)".to_string()
    } else {
        bps.iter()
            .filter(|(_, _, enabled)| *enabled)
            .map(|(id, addr, _)| format!("{}: 0x{:04X}", id, addr))
            .collect::<Vec<_>>()
            .join("\n")
    };
    Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Breakpoints "))
}

fn render_register_modal(reg: SelectedRegister, gb: &GameBoy, area: Rect) -> (Rect, Paragraph<'static>) {
    let value = reg.value(gb);
    let (name, hi_name, lo_name) = reg.names();
    let hi = (value >> 8) as u8;
    let lo = (value & 0xFF) as u8;

    let label_style = Style::default().fg(Color::Cyan);
    let header_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default();

    let sep_style = Style::default().fg(Color::DarkGray);
    let sep = Span::styled("│", sep_style);

    // Fixed column widths: Dec=7, Hex=8, Bin=18
    let lines = vec![
        Line::from(vec![
            Span::styled("         ", label_style),
            Span::styled("   Dec ", header_style),
            sep.clone(),
            Span::styled("   Hex  ", header_style),
            sep.clone(),
            Span::styled("        Bin       ", header_style),
        ]),
        Line::from(vec![
            Span::styled("   Full: ", label_style),
            Span::styled(format!("{:>6} ", value), value_style),
            sep.clone(),
            Span::styled(format!(" 0x{:04X} ", value), value_style),
            sep.clone(),
            Span::styled(format!(" 0b{:016b} ", value), value_style),
        ]),
        Line::from(vec![
            Span::styled(format!(" Hi ({}): ", hi_name), label_style),
            Span::styled(format!("{:>6} ", hi), value_style),
            sep.clone(),
            Span::styled(format!("   {:02X}   ", hi), value_style),
            sep.clone(),
            Span::styled(format!("   {:08b}         ", hi), value_style),
        ]),
        Line::from(vec![
            Span::styled(format!(" Lo ({}): ", lo_name), label_style),
            Span::styled(format!("{:>6} ", lo), value_style),
            sep.clone(),
            Span::styled(format!("     {:02X} ", lo), value_style),
            sep.clone(),
            Span::styled(format!("           {:08b} ", lo), value_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Esc or click to close", Style::default().fg(Color::DarkGray))),
    ];

    // Calculate centered modal area
    let modal_width = 56;
    let modal_height = 8;
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    let title = format!(" {} ", name);
    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Yellow)));

    (modal_area, paragraph)
}

fn ui(frame: &mut Frame, debugger: &mut TuiDebugger, gb: &GameBoy) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Top row (registers + flags)
            Constraint::Min(8),     // Middle (memory + breakpoints) - now larger
            Constraint::Length(3),  // Input with status in title
        ])
        .split(frame.size());

    // Top row: Registers | Flags
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(main_layout[0]);

    // Save registers area for click detection
    debugger.registers_area = top_row[0];
    frame.render_widget(render_registers(gb.cpu()), top_row[0]);
    frame.render_widget(render_flags(gb.cpu()), top_row[1]);

    // Middle row: Memory | Breakpoints
    let mid_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(main_layout[1]);

    // Calculate how many lines we can show in the memory area
    // Account for borders (2 lines)
    let memory_height = mid_row[0].height.saturating_sub(2);
    debugger.memory_lines = memory_height;

    // Calculate bytes per line based on available width
    // Format: "XXXX │ XX XX XX XX  XX XX XX XX │ ........"
    // Per byte: 3 chars (hex + space) + 1 char (ascii) = 4 chars
    // Fixed: 7 (addr + " │ ") + 3 (" │ ") + 2 (borders) = 12 chars
    // Extra space at midpoint: 1 char
    let available_width = mid_row[0].width.saturating_sub(13) as u16;
    // Each byte needs ~4 chars total
    let max_bytes = available_width / 4;
    // Pick the largest option that fits
    let bytes_per_line = BYTES_PER_LINE_OPTIONS
        .iter()
        .rev()
        .find(|&&b| b <= max_bytes)
        .copied()
        .unwrap_or(BYTES_PER_LINE_OPTIONS[0]);
    debugger.bytes_per_line = bytes_per_line;

    let memory_focused = debugger.focus == Focus::Memory;
    frame.render_widget(
        render_memory(gb, debugger.memory_offset, memory_height, bytes_per_line, memory_focused, &debugger.highlight),
        mid_row[0]
    );
    frame.render_widget(render_breakpoints(&debugger.breakpoints), mid_row[1]);

    // Command input with status in title
    debugger.update_input_block();
    frame.render_widget(&debugger.textarea, main_layout[2]);

    // Render modal on top if open
    if let Some(reg) = debugger.register_modal {
        let (modal_area, modal_widget) = render_register_modal(reg, gb, frame.size());
        frame.render_widget(Clear, modal_area);
        frame.render_widget(modal_widget, modal_area);
    }
}

/// Run the TUI debugger
pub fn run(gb: &mut GameBoy) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut debugger = TuiDebugger::new();
    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|frame| ui(frame, &mut debugger, gb))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    let input = Input::from(key);
                    if !debugger.handle_input(input, gb) {
                        should_quit = true;
                    }
                }
                Event::Mouse(mouse) => {
                    debugger.handle_mouse(mouse, gb);
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}
