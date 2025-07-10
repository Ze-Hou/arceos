use core::ptr::NonNull;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::register_structs;

register_structs! {
    WatchDogRegs {
        (0x00 => vid: ReadOnly<u16>), // Vendor ID Register
        (0x02 => did: ReadOnly<u16>), // Device ID Register
        (0x04 => com: ReadWrite<u16>), // Command Register
        (0x06 => ds: ReadWrite<u16>), // Device Status Register
        (0x08 => rid: ReadOnly<u8>), // Revision ID Register
        (0x09 => pi: ReadOnly<u8>), // Programming Interface Register
        (0x0A => scc: ReadOnly<u8>), // Sub Class Code Register
        (0x0B => bcc: ReadOnly<u8>), // Base Class Code Register 
        (0x0C => _reserved0c), // Reserved
        (0x0E => hedt: ReadOnly<u8>), // Header Type Register
        (0x0F => _reserved0f), // Reserved
        (0x10 => bar: ReadWrite<u32>), // Base Address Register 
        (0x14 => _reserved14), // Reserved
        (0x2C => svid: ReadWrite<u16>), // Subsystem Vendor ID Register
        (0x2E => sid: ReadWrite<u16>), // Subsystem ID Register
        (0x30 => _reserved30), // Reserved
        (0x60 => wdt: ReadWrite<u16>),
        (0x62 => _reserved62), // Reserved
        (0x68 => wdt_lock: ReadWrite<u8>),
        (0x69 => _reserved69), // Reserved
        (0xF8 => mid: ReadOnly<u32>), // Manufacturer’s ID Register
        (0xFc => _reservedfc), // Reserved
        (0x100 => @END),
    }
}

register_structs! {
    MemMapRegs {
        (0x00 => preload_value_1: ReadWrite<u32>),
        (0x04 => preload_value_2: ReadWrite<u32>),
        (0x08 => wdt_int_status: ReadWrite<u32>),
        (0x0C => wdt_load: WriteOnly<u32>),
        (0x10 => @END),
    }
}

struct WatchDog_Config {
    base: NonNull<WatchDogRegs>,
}

struct WatchDog {
    base: NonNull<MemMapRegs>,
}

unsafe impl Send for WatchDog_Config {}
unsafe impl Sync for WatchDog_Config {}

unsafe impl Send for WatchDog {}
unsafe impl Sync for WatchDog {}

impl WatchDog_Config {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &WatchDogRegs {
        unsafe { self.base.as_ref() }
    }

    pub fn id(&self) {
        let vendor_id = self.regs().vid.get() as u32;
        let device_id = self.regs().did.get() as u32;
        let id = (vendor_id << 16) | device_id;
        info!("Watchdog ID: {:#010x}", id);
    }

    pub fn set_bar(&mut self) {
        self.regs().bar.set(0x10008000);
        info! {"Watchdog BAR set to {:#x}", self.regs().bar.get()};
    }

    pub fn enable_mem(&mut self) {
        let mut com = self.regs().com.get();
        info! {"Current Command Register: {:#x}", com};
        com |= 0x0002; // Enable memory-mapped access
        self.regs().com.set(com);
        info! {"Updated Command Register: {:#x}", self.regs().com.get()};
        info! {"Watchdog memory-mapped access enabled"};
    }

    pub fn set_wdt_lock(&mut self) {
        let mut wdt_lock = self.regs().wdt_lock.get();
        wdt_lock |= 0x02;
        self.regs().wdt_lock.set(wdt_lock);
        info! {"Watchdog enable"};
    }
    
}

impl WatchDog {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &MemMapRegs {
        unsafe { self.base.as_ref() }
    }

    pub fn read_preload_value_1(&self) {
        info! {"Preload Value 1: {:#x}", self.regs().preload_value_1.get()}
    }

    pub fn read_preload_value_2(&self) {
        info! {"Preload Value 2: {:#x}", self.regs().preload_value_2.get()}
    }

    pub fn read_wdt_int_status(&self) {
        info! {"Watchdog Interrupt Status: {:#x}", self.regs().wdt_int_status.get()}
    }

    pub fn unlock_wdt(&mut self) {
        self.regs().wdt_load.set(0x80);
        self.regs().wdt_load.set(0x86);
        info! {"Watchdog unlocked"};
    }

    pub fn set_preload_value_1(&mut self, value: u32) {
        self.unlock_wdt();
        self.regs().preload_value_1.set(value);
        info! {"Set Preload Value 1 to: {}", self.regs().preload_value_1.get()};
    }

    pub fn set_preload_value_2(&mut self, value: u32) {
        self.unlock_wdt();
        self.regs().preload_value_2.set(value);
        info! {"Set Preload Value 2 to: {}", self.regs().preload_value_2.get()};
    }

    pub fn reset(&mut self) {
        self.unlock_wdt();
        self.regs().wdt_load.set(0x0100);
        info! {"Watchdog reset with preload value set to 0x0100"};
    }
}

use memory_addr::PhysAddr;
use kspin::SpinNoIrq;
use crate::mem::phys_to_virt;

const PCI_CONFIG_BASE: usize = (axconfig::devices::PCI_ECAM_BASE as usize) + (2 << 15);
const WDT_CONFIG_BASE: PhysAddr = pa!(PCI_CONFIG_BASE);

static WDT_CONFIG: SpinNoIrq<WatchDog_Config> = SpinNoIrq::new(
    WatchDog_Config::new(phys_to_virt(WDT_CONFIG_BASE).as_mut_ptr())
);

const WDT_BASE: PhysAddr = pa!(axconfig::devices::WDT_PADDR);

static WDT: SpinNoIrq<WatchDog> = SpinNoIrq::new(
    WatchDog::new(phys_to_virt(WDT_BASE).as_mut_ptr())
);

const WDT_IRQ_NUM: usize = crate::platform::irq::WDT_IRQ_NUM;

pub fn watchdog_init() {
    info! {"WDT_IRQ_NUM: {}", WDT_IRQ_NUM};

    info! {"Initializing Watchdog at {:#x}", WDT_CONFIG_BASE};
    info! {"Watchdog Memory Base: {:#x}", WDT_BASE};
    WDT_CONFIG.lock().id();
    WDT_CONFIG.lock().set_bar();
    WDT_CONFIG.lock().enable_mem();
    WDT_CONFIG.lock().set_wdt_lock();
    WDT.lock().reset();
    WDT.lock().read_preload_value_1();
    WDT.lock().read_preload_value_2();
    WDT.lock().read_wdt_int_status();
    
}


