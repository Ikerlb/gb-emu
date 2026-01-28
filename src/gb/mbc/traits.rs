/// Core traits for MBC implementations

/// Memory operations - read and write bytes
pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

/// Persistence - for battery-backed cartridges
#[allow(dead_code)] // Infrastructure for future save file support
pub trait Stable {
    /// Returns data to save to a .sav file
    fn save_data(&self) -> Vec<u8>;

    /// Loads data from a .sav file
    fn load_data(&mut self, data: &[u8]);
}

/// Combined trait for all MBC functionality
pub trait Mbc: Memory + Stable + Send {}

/// Blanket impl: anything implementing Memory + Stable + Send is an Mbc
impl<T: Memory + Stable + Send> Mbc for T {}
