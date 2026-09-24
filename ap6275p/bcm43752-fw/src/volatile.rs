#[inline(always)]
pub unsafe fn read8(addr: usize) -> u8 { unsafe { core::ptr::read_volatile(addr as *const u8) } }
#[inline(always)]
pub unsafe fn read32(addr: usize) -> u32 { unsafe { core::ptr::read_volatile(addr as *const u32) } }
#[inline(always)]
pub unsafe fn write8(addr: usize, value: u8) { unsafe { core::ptr::write_volatile(addr as *mut u8, value) } }
#[inline(always)]
pub unsafe fn write32(addr: usize, value: u32) { unsafe { core::ptr::write_volatile(addr as *mut u32, value) } }
