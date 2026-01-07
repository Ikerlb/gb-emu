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
    pub fn new() -> Self {
        Self::default()
    }

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
