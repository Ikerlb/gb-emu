use crate::gb::interconnect::Interconnect;
use std::fmt::Write;
use std::str::FromStr;

/// Configuration for debug output and emulator behavior
#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    /// Enable debug output after each instruction
    pub enabled: bool,
    /// Use verbose multi-line format instead of compact
    pub verbose: bool,
    /// Maximum number of instructions to execute (None = unlimited)
    pub max_instructions: Option<u64>,
}

impl DebugConfig {
    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_max_instructions(mut self, max_instructions: Option<u64>) -> Self {
        self.max_instructions = max_instructions;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRange {
    pub start: u16,
    pub end: u16,
}

impl MemoryRange {
    /// Returns true if the address falls within this range (inclusive)
    pub fn contains(&self, addr: u16) -> bool {
        addr >= self.start && addr <= self.end
    }
}

impl FromStr for MemoryRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s_raw, e_raw) = s
            .split_once(':')
            .ok_or_else(|| format!("Invalid range format: '{}'. Expected START:END", s))?;

        let (s_str, e_str) = (s_raw.trim(), e_raw.trim());

        let start = parse_address(s_str)?;
        let end = parse_address(e_str)?;

        if start > end {
            return Err(format!(
                "End address '{}' must be >= start address '{}'",
                e_str, s_str
            ));
        }

        Ok(Self { start, end })
    }
}

/// Parses an address string (hex with 0x/0X, bin with 0b/0B prefix or decimal)
pub fn parse_address(s: &str) -> Result<u16, String> {
    let s = s.trim();

    let (input, radix) = match s.get(..2) {
        Some("0x") | Some("0X") => (&s[2..], 16),
        Some("0b") | Some("0B") => (&s[2..], 2),
        _ => (s, 10),
    };

    u16::from_str_radix(input, radix)
        .map_err(|e| format!("Invalid address '{}': {}", s, e))
}

/// Represents a byte read result for hex dump formatting
enum ByteRead {
    OutOfRange,
    Unimplemented,
    Value(u8),
}

impl ByteRead {
    /// Writes the 2-char hex representation to the given buffer
    fn write_hex(&self, buf: &mut String) {
        match self {
            ByteRead::Value(b) => write!(buf, "{:02X}", b).unwrap(),
            ByteRead::Unimplemented => buf.push_str("??"),
            ByteRead::OutOfRange => buf.push_str("  "),
        }
    }

    /// Returns the ASCII character representation for this byte
    fn as_ascii(&self) -> char {
        match self {
            ByteRead::Value(b) if (0x20..=0x7E).contains(b) => *b as char,
            ByteRead::Value(_) => '.',
            ByteRead::Unimplemented => '?',
            ByteRead::OutOfRange => ' ',
        }
    }
}

/// Formats a hex dump of memory from start to end (inclusive)
pub fn format_memory_dump(interconnect: &Interconnect, range: MemoryRange) -> String {
    let aligned_start = range.start & 0xFFF0;

    (aligned_start..=range.end)
        .step_by(16)
        .map(|line_addr| {
            let bytes: Vec<ByteRead> = (0..16)
                .map(|offset| {
                    let addr = line_addr.wrapping_add(offset);
                    if !range.contains(addr) {
                        ByteRead::OutOfRange
                    } else {
                        interconnect
                            .try_read(addr)
                            .map_or(ByteRead::Unimplemented, ByteRead::Value)
                    }
                })
                .collect();

            let hex_part = format_hex_line(&bytes);
            let ascii_part: String = bytes.iter().map(ByteRead::as_ascii).collect();

            format!("{:04X}  {} |{}|", line_addr, hex_part, ascii_part)
        })
        .collect::<Vec<_>>()
        .join("\n") + "\n"
}

/// Formats 16 bytes as a hex line with a separator after byte 7
fn format_hex_line(bytes: &[ByteRead]) -> String {
    let mut result = String::with_capacity(50);
    for (i, byte) in bytes.iter().enumerate() {
        byte.write_hex(&mut result);
        result.push_str(if i == 7 { "  " } else { " " });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_hex() {
        assert_eq!(parse_address("0x0000"), Ok(0x0000));
        assert_eq!(parse_address("0x00FF"), Ok(0x00FF));
        assert_eq!(parse_address("0xFFFF"), Ok(0xFFFF));
        assert_eq!(parse_address("0X1234"), Ok(0x1234));
    }

    #[test]
    fn test_parse_address_decimal() {
        assert_eq!(parse_address("0"), Ok(0));
        assert_eq!(parse_address("255"), Ok(255));
        assert_eq!(parse_address("65535"), Ok(65535));
    }

    #[test]
    fn test_parse_address_with_whitespace() {
        assert_eq!(parse_address("  0x100  "), Ok(0x100));
        assert_eq!(parse_address("\t256\n"), Ok(256));
    }

    #[test]
    fn test_parse_address_invalid() {
        assert!(parse_address("0xGGGG").is_err());
        assert!(parse_address("not_a_number").is_err());
        assert!(parse_address("0x10000").is_err());
        assert!(parse_address("65536").is_err());
    }

    #[test]
    fn test_memory_range_from_str_valid() {
        assert_eq!(
            "0x0000:0x00FF".parse::<MemoryRange>(),
            Ok(MemoryRange { start: 0x0000, end: 0x00FF })
        );
        assert_eq!(
            "0:255".parse::<MemoryRange>(),
            Ok(MemoryRange { start: 0, end: 255 })
        );
        assert_eq!(
            "0x100:512".parse::<MemoryRange>(),
            Ok(MemoryRange { start: 0x100, end: 512 })
        );
        assert_eq!(
            "0x0000:0x0000".parse::<MemoryRange>(),
            Ok(MemoryRange { start: 0, end: 0 })
        );
    }

    #[test]
    fn test_memory_range_from_str_invalid() {
        assert!("0x0000-0x00FF".parse::<MemoryRange>().is_err());
        assert!("0x00FF:0x0000".parse::<MemoryRange>().is_err());
        assert!("abc:def".parse::<MemoryRange>().is_err());
        assert!("0:1:2".parse::<MemoryRange>().is_err());
    }

    #[test]
    fn test_memory_range_contains() {
        let range = MemoryRange { start: 0x100, end: 0x1FF };
        assert!(!range.contains(0x00));
        assert!(!range.contains(0xFF));
        assert!(range.contains(0x100));
        assert!(range.contains(0x150));
        assert!(range.contains(0x1FF));
        assert!(!range.contains(0x200));
    }
}
