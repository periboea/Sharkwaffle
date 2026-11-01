//! The Cortex-M4 CPU state and the boot sequence.

use crate::bus::{Bus, BusFault, FLASH_BASE};

pub const SP: usize = 13;
pub const LR: usize = 14;
pub const PC: usize = 15;

/// Bit 24 of xPSR is the Thumb bit. Cortex-M only runs Thumb code, so it is always 1.
const XPSR_THUMB: u32 = 1 << 24;

pub struct Cpu {
    /// r0 to r12 are general purpose, r13 is SP, r14 is LR, r15 is PC.
    pub r: [u32; 16],
    pub xpsr: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            r: [0; 16],
            xpsr: XPSR_THUMB,
        }
    }

    /// What the chip does when power comes on.
    /// Word 0 of the vector table is the starting stack pointer.
    /// Word 1 is the address of the reset handler.
    pub fn reset(&mut self, bus: &Bus) -> Result<(), BusFault> {
        let initial_sp = bus.read32(FLASH_BASE)?;
        let reset_vector = bus.read32(FLASH_BASE + 4)?;

        self.r = [0; 16];
        self.r[SP] = initial_sp & !0b11; // The ARM CPU crashes or gets very unhappy if the Stack Pointer isn't aligned to a 4-byte boundary. so yeah this rounds the initial Stack Pointer (SP) address down to the nearest multiple of 4.
        self.r[LR] = 0xFFFF_FFFF;
        // Bit 0 of a vector is the Thumb flag, not part of the address. Strip it.
        self.r[PC] = reset_vector & !1;
        self.xpsr = XPSR_THUMB;
        Ok(())
    }

    pub fn fetch_instructions(&mut self, bus: &Bus) -> Result<u16, BusFault> {
        let opcode = bus.read16(self.r[PC])?;
        self.r[PC] += 2;
        Ok(opcode)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}