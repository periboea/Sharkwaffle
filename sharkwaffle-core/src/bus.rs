//! The memory bus. Every read and write the CPU does goes through here.
//! addresses and sizes are from RM0390!!!!

pub const FLASH_BASE: u32 = 0x0800_0000;
pub const FLASH_SIZE: usize = 512 * 1024;
pub const SRAM_BASE: u32 = 0x2000_0000;
pub const SRAM_SIZE: usize = 128 * 1024;

/// Something went wrong talking to memory.
#[derive(Debug, PartialEq, Eq)]
pub enum BusFault {
    /// Nothing lives at this address (yet).
    Unmapped(u32),
    /// The CPU tries to write directly to flash. Flash memory can't be overwritten with a simple store operation.
    FlashWrite(u32),
    /// The firmware image is bigger than the flash.
    ImageTooBig,
}

pub struct Bus {
    flash: Vec<u8>,
    sram: Vec<u8>,
}

impl Bus {
    pub fn new() -> Self {
        // Real erased flash reads as 0xFF, so we copy that.
        Bus {
            flash: vec![0xFF; FLASH_SIZE],
            sram: vec![0; SRAM_SIZE],
        }
    }

    /// Copy a raw .bin firmware into flash, starting at the very beginning.
    pub fn load_flash(&mut self, image: &[u8]) -> Result<(), BusFault> {
        if image.len() > FLASH_SIZE {
            return Err(BusFault::ImageTooBig);
        }
        self.flash[..image.len()].copy_from_slice(image);
        Ok(())
    }

    /// Find which memory block an address lands in, and where inside it.
    fn locate(&self, addr: u32, len: usize) -> Result<(Region, usize), BusFault> {
        let a = addr as usize;
        let flash_start = FLASH_BASE as usize;
        let sram_start = SRAM_BASE as usize;
        if a >= flash_start && a + len <= flash_start + FLASH_SIZE {
            Ok((Region::Flash, a - flash_start))
        } else if a >= sram_start && a + len <= sram_start + SRAM_SIZE {
            Ok((Region::Sram, a - sram_start))
        } else {
            Err(BusFault::Unmapped(addr))
        }
    }

    fn read_bytes(&self, addr: u32, len: usize) -> Result<&[u8], BusFault> {
        let (region, off) = self.locate(addr, len)?;
        let mem = match region {
            Region::Flash => &self.flash,
            Region::Sram => &self.sram,
        };
        Ok(&mem[off..off + len])
    }

    fn write_bytes(&mut self, addr: u32, data: &[u8]) -> Result<(), BusFault> {
        let (region, off) = self.locate(addr, data.len())?;
        match region {
            // in real hardware, Flash memory can't be overwritten with a simple store operation.
            Region::Flash => Err(BusFault::FlashWrite(addr)),
            Region::Sram => {
                self.sram[off..off + data.len()].copy_from_slice(data);
                Ok(())
            }
        }
    }

    // The STM32 is little endian: the low byte lives at the lowest address.
    pub fn read8(&self, addr: u32) -> Result<u8, BusFault> {
        Ok(self.read_bytes(addr, 1)?[0])
    }
    pub fn read16(&self, addr: u32) -> Result<u16, BusFault> {
        let b = self.read_bytes(addr, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn read32(&self, addr: u32) -> Result<u32, BusFault> {
        let b = self.read_bytes(addr, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn write8(&mut self, addr: u32, v: u8) -> Result<(), BusFault> {
        self.write_bytes(addr, &[v])
    }
    pub fn write16(&mut self, addr: u32, v: u16) -> Result<(), BusFault> {
        self.write_bytes(addr, &v.to_le_bytes())
    }
    pub fn write32(&mut self, addr: u32, v: u32) -> Result<(), BusFault> {
        self.write_bytes(addr, &v.to_le_bytes())
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

enum Region {
    Flash,
    Sram,
}