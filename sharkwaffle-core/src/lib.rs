pub mod bus;
pub mod cpu;

#[cfg(test)]
mod tests {
    use object::ReadCacheOps;
    use crate::bus::*;
    use crate::cpu::*;

    /// Build a tiny fake firmware, just a vector table with SP and reset vector.
    fn fake_image() -> Vec<u8> {
        let mut img = Vec::new();
        img.extend_from_slice(&0x2002_0000u32.to_le_bytes()); // top of SRAM
        img.extend_from_slice(&0x0800_0009u32.to_le_bytes()); // reset handler, thumb bit set
        img.extend_from_slice(&0x0101u16.to_le_bytes());
        img
    }

    #[test]
    fn boot_reads_vector_table() {
        let mut bus = Bus::new();
        bus.load_flash(&fake_image()).unwrap();
        let mut cpu = Cpu::new();
        cpu.reset(&bus).unwrap();
        assert_eq!(cpu.r[SP], 0x2002_0000);
        assert_eq!(cpu.r[PC], 0x0800_0008); // thumb bit stripped
        assert_eq!(cpu.r[LR], 0xFFFF_FFFF);
    }

    #[test]
    fn sram_round_trip_is_little_endian() {
        let mut bus = Bus::new();
        bus.write32(SRAM_BASE, 0xDEAD_BEEF).unwrap();
        assert_eq!(bus.read8(SRAM_BASE).unwrap(), 0xEF);
        assert_eq!(bus.read16(SRAM_BASE + 2).unwrap(), 0xDEAD);
        assert_eq!(bus.read32(SRAM_BASE).unwrap(), 0xDEAD_BEEF);
    }

    #[test]
    fn flash_is_read_only_and_holes_fault() {
        let mut bus = Bus::new();
        assert_eq!(bus.write32(FLASH_BASE, 1), Err(BusFault::FlashWrite(FLASH_BASE)));
        assert_eq!(bus.read32(0x4002_0000), Err(BusFault::Unmapped(0x4002_0000)));
        assert_eq!(bus.read32(SRAM_BASE + SRAM_SIZE as u32 - 2), Err(BusFault::Unmapped(SRAM_BASE + SRAM_SIZE as u32 - 2)));
    }

    // TODO test still needs fixing. I lost the logic of what the fuck bus supposed to return.
    #[test]
    fn fetch_instructions_works_and_advances_pc() {
        let mut bus = Bus::new();
        let fake_image = fake_image();
        bus.load_flash(&fake_image).unwrap();

        let mut cpu = Cpu::new();
        cpu.reset(&bus).unwrap();

        let instruction = cpu.fetch_instructions(&bus).unwrap();

        assert_eq!(instruction, 0x0101);
        assert_eq!(cpu.r[PC], 0x0800_0008 + 2);
    }
}