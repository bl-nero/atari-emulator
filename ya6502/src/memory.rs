use std::error;
use std::fmt;
use std::result::Result;

pub trait Read {
    /// Reads a byte from given address. Returns the byte or error if the
    /// location is unsupported. In a release build, the errors should be
    /// ignored and the method should always return a successful result.
    fn read(&self, address: u16) -> ReadResult;
}

pub trait Write {
    /// Writes a byte to given address. Returns error if the location is
    /// unsupported. In a release build, the errors should be ignored and the
    /// method should always return a successful result.
    fn write(&mut self, address: u16, value: u8) -> WriteResult;
}

pub trait Memory: Read + Write {}

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

/// A very simple memory structure: just a 64-kilobyte chunk of RAM.
pub struct SimpleRam {
    pub bytes: [u8; Self::SIZE],
}

impl SimpleRam {
    const SIZE: usize = 0x10000; // 64 kB (64 * 1024)

    pub fn new() -> SimpleRam {
        SimpleRam {
            bytes: [0; Self::SIZE], // Fill the entire RAM with 0x00.
        }
    }

    pub fn initialized_with(value: u8) -> SimpleRam {
        SimpleRam {
            bytes: [value; Self::SIZE],
        }
    }

    /// Creates a new `RAM`, putting given `program` at address 0xF000. It also
    /// sets the reset pointer to 0xF000.
    pub fn with_test_program(program: &[u8]) -> SimpleRam {
        Self::with_test_program_at(0xF000, program)
    }

    /// Creates a new `RAM`, putting given `program` at a given address. It also
    /// sets the reset pointer to this address.
    pub fn with_test_program_at(address: u16, program: &[u8]) -> SimpleRam {
        let mut ram = SimpleRam::new();
        ram.bytes[address as usize..address as usize + program.len()].copy_from_slice(program);
        ram.bytes[0xFFFC] = address as u8; // least-significant byte
        ram.bytes[0xFFFD] = (address >> 8) as u8; // most-significant byte
        return ram;
    }
}

impl Read for SimpleRam {
    fn read(&self, address: u16) -> ReadResult {
        // this arrow means we give u16 they return u8
        Ok(self.bytes[address as usize])
    }
}

impl Write for SimpleRam {
    fn write(&mut self, address: u16, value: u8) -> WriteResult {
        self.bytes[address as usize] = value;
        Ok(())
    }
}

impl Memory for SimpleRam {}

impl fmt::Debug for SimpleRam {
    /// Prints out only the zero page, because come on, who would scroll through
    /// a dump of entire 64 kibibytes...
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::convert::TryInto;
        let zero_page: [u8; 255] = (&self.bytes[..255]).try_into().unwrap();
        return f
            .debug_struct("SimpleRam")
            .field("zero page", &zero_page)
            .finish();
    }
}

// A 128-byte memory structure that acts as Atari RAM and supports memory space
// mirroring.
#[derive(Debug)]
pub struct AtariRam {
    bytes: [u8; Self::SIZE],
}

impl AtariRam {
    const SIZE: usize = 0x80;
    pub fn new() -> AtariRam {
        AtariRam {
            bytes: [0; Self::SIZE],
        }
    }
}

impl Read for AtariRam {
    fn read(&self, address: u16) -> ReadResult {
        Ok(self.bytes[address as usize & 0b0111_1111])
    }
}

impl Write for AtariRam {
    fn write(&mut self, address: u16, value: u8) -> WriteResult {
        self.bytes[address as usize & 0b0111_1111] = value;
        Ok(())
    }
}

impl Memory for AtariRam {}

#[derive(Debug, Clone, PartialEq)]
pub struct RomSizeError {
    size: usize,
}
impl error::Error for RomSizeError {}
impl fmt::Display for RomSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Illegal ROM size: {} bytes. Valid sizes: 2048, 4096",
            self.size
        )
    }
}

pub struct AtariRom {
    bytes: Vec<u8>,
    address_mask: u16,
}

impl AtariRom {
    pub fn new(bytes: &[u8]) -> Result<Self, RomSizeError> {
        match bytes.len() {
            2048 | 4096 => Ok(AtariRom {
                bytes: bytes.to_vec(),
                address_mask: match bytes.len() {
                    0x1000 => 0b0000_1111_1111_1111,
                    _ => 0b0000_0111_1111_1111,
                },
            }),
            _ => Err(RomSizeError { size: bytes.len() }),
        }
    }
}

impl Read for AtariRom {
    fn read(&self, address: u16) -> ReadResult {
        Ok(self.bytes[(address & self.address_mask) as usize])
    }
}

impl fmt::Debug for AtariRom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        f.debug_struct("AtariRom")
            .field("size", &self.bytes.len())
            .field("address_mask", &self.address_mask)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_empty_simple_ram() {
        let ram = SimpleRam::with_test_program(&[]);
        assert_eq!(ram.bytes[..0xFFFC], [0u8; 0xFFFC][..]);
    }

    #[test]
    fn simple_ram_with_test_program() {
        let ram = SimpleRam::with_test_program(&[10, 56, 72, 255]);
        // Bytes until 0xF000 (exclusively) should have been zeroed.
        assert_eq!(ram.bytes[..0xF000], [0u8; 0xF000][..]);
        // Next, there should be our program.
        assert_eq!(ram.bytes[0xF000..0xF004], [10, 56, 72, 255][..]);
        // The rest, until 0xFFFC, should also be zeroed.
        assert_eq!(ram.bytes[0xF004..0xFFFC], [0u8; 0xFFFC - 0xF004][..]);
        // And finally, the reset vector.
        assert_eq!(ram.bytes[0xFFFC..0xFFFE], [0x00, 0xF0]);
    }

    #[test]
    fn simple_ram_with_test_program_at() {
        let ram = SimpleRam::with_test_program_at(0xF110, &[10, 56, 72, 255]);
        assert_eq!(ram.bytes[..0xF110], [0u8; 0xF110][..]);
        assert_eq!(ram.bytes[0xF110..0xF114], [10, 56, 72, 255][..]);
        assert_eq!(ram.bytes[0xF114..0xFFFC], [0u8; 0xFFFC - 0xF114][..]);
        assert_eq!(ram.bytes[0xFFFC..0xFFFE], [0x10, 0xF1]);
    }

    #[test]
    fn simple_ram_with_test_program_sets_reset_address() {
        let ram = SimpleRam::with_test_program(&[0xFF; 0x1000]);
        assert_eq!(ram.bytes[0xFFFC..0xFFFE], [0x00, 0xF0]); // 0xF000
    }

    #[test]
    fn atari_ram_read_write() {
        let mut ram = AtariRam::new();
        ram.write(0x00AB, 123).unwrap();
        ram.write(0x00AC, 234).unwrap();
        assert_eq!(ram.read(0x00AB).unwrap(), 123);
        assert_eq!(ram.read(0x00AC).unwrap(), 234);
    }

    #[test]
    fn atari_ram_mirroring() {
        let mut ram = AtariRam::new();
        ram.write(0x0080, 1).unwrap();
        assert_eq!(ram.read(0x0080).unwrap(), 1);
        assert_eq!(ram.read(0x2880).unwrap(), 1);
        assert_eq!(ram.read(0xCD80).unwrap(), 1);
    }

    #[test]
    fn atari_rom_4k() {
        let mut program = [0u8; 0x1000];
        program[5] = 1;
        let rom = AtariRom::new(&program).unwrap();
        assert_eq!(rom.read(0x1000).unwrap(), 0);
        assert_eq!(rom.read(0x1005).unwrap(), 1);
        assert_eq!(rom.read(0x3005).unwrap(), 1);
        assert_eq!(rom.read(0xF005).unwrap(), 1);
    }

    #[test]
    fn atari_rom_2k() {
        let mut program = [0u8; 0x0800];
        program[5] = 1;
        let rom = AtariRom::new(&program).unwrap();
        assert_eq!(rom.read(0x1000).unwrap(), 0);
        assert_eq!(rom.read(0x1005).unwrap(), 1);
        assert_eq!(rom.read(0x3005).unwrap(), 1);
        assert_eq!(rom.read(0xF005).unwrap(), 1);
        assert_eq!(rom.read(0xF805).unwrap(), 1);
    }

    #[test]
    fn atari_rom_illegal_size() {
        let rom = AtariRom::new(&[0u8; 0x0900]);
        assert_eq!(rom.err(), Some(RomSizeError { size: 0x900 }));
    }
}
