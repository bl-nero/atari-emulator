use std::fmt;
use std::error;
use std::result::Result;

const RAM_SIZE: usize = 0x10000; // 64 kB (64 * 1024)

pub trait Memory {
    /// Writes a byte to given address. Returns error if the location is
    /// unsupported. In a release build, the errors should be ignored and the
    /// method should always return a successful result.
    fn write(&mut self, address: u16, value: u8) -> WriteResult;

    /// Reads a byte from given address. Returns the byte or error if the
    /// location is unsupported. In a release build, the errors should be
    /// ignored and the method should always return a successful result.
    fn read(&self, address: u16) -> ReadResult;
}

pub type ReadResult = Result<u8, ReadError>;

#[derive(Debug, Clone)]
pub struct ReadError {
    pub address: u16,
}

impl error::Error for ReadError {}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to read from address ${:04X}", self.address)
    }
}

pub type WriteResult = Result<(), WriteError>;

#[derive(Debug, Clone)]
pub struct WriteError {
    pub address: u16,
    pub value: u8,
}

impl error::Error for WriteError {}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unable to write ${:02X} to address ${:04X}",
            self.value, self.address
        )
    }
}

/// A very simple memory structure. At the moment, it's just a 64-kilobyte chunk
/// of RAM, for simplicity of addressing.
pub struct RAM {
    pub bytes: [u8; RAM_SIZE], // computer has RAM_SIZE (64k) bytes (unsigned 8-bit integers)
}

impl RAM {
    pub fn new() -> RAM {
        RAM {
            bytes: [0; RAM_SIZE], // Fill the entire RAM with 0x00.
        }
    }

    pub fn with_program(program: &[u8]) -> RAM {
        let mut ram = RAM::new();

        // Copy the program into memory. If the program is a 2K cartridge, place
        // it in two mirror copies, starting from addresses 0xF000 and 0xF800.
        for (i, byte) in program.iter().enumerate() {
            ram.bytes[0xF000 + i] = *byte;
            if program.len() == 0x800 {
                ram.bytes[0xF800 + i] = *byte;
            }
        }
        return ram;
    }

    /// Creates a new `RAM`, putting given `program` at address 0xF000. It also
    /// sets the reset pointer to 0xF000.
    pub fn with_test_program(program: &[u8]) -> RAM {
        let mut ram = RAM::new();

        // Copy the program into memory, starting from address 0xF000.
        for (i, byte) in program.iter().enumerate() {
            ram.bytes[0xF000 + i] = *byte;
        }

        // Initialize the reset address (stored at 0xFFFC) to 0xF000.
        ram.bytes[0xFFFC] = 0x00; // least-significant byte
        ram.bytes[0xFFFD] = 0xF0; // most-significant byte
        return ram;
    }
}

impl Memory for RAM {
    fn read(&self, address: u16) -> ReadResult {
        // this arrow means we give u16 they return u8
        Ok(self.bytes[address as usize])
    }

    fn write(&mut self, address: u16, value: u8) -> WriteResult {
        self.bytes[address as usize] = value;
        Ok(())
    }
}

impl fmt::Debug for RAM {
    /// Prints out only the zero page, because come on, who would scroll through
    /// a dump of entire 64 kibibytes...
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::convert::TryInto;
        let zero_page: [u8; 255] = (&self.bytes[..255]).try_into().unwrap();
        return f
            .debug_struct("RAM")
            .field("zero page", &zero_page)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_empty_ram() {
        let ram = RAM::with_test_program(&[]);
        assert_eq!(ram.bytes[..0xFFFC], [0u8; 0xFFFC][..]);
    }

    #[test]
    fn it_places_program_in_memory() {
        let ram = RAM::with_test_program(&[10, 56, 72, 255]);
        // Bytes until 0xF000 (exclusively) should have been zeroed.
        assert_eq!(ram.bytes[..0xF000], [0u8; 0xF000][..]);
        // Next, there should be our program.
        assert_eq!(ram.bytes[0xF000..0xF004], [10, 56, 72, 255][..]);
        // The rest, until 0xFFFC, should also be zeroed.
        assert_eq!(ram.bytes[0xF004..0xFFFC], [0u8; 0xFFFC - 0xF004][..]);
    }

    #[test]
    fn it_sets_reset_address() {
        let ram = RAM::with_test_program(&[0xFF; 0x1000]);
        assert_eq!(ram.bytes[0xFFFC..0xFFFE], [0x00, 0xF0]); // 0xF000
    }
}
