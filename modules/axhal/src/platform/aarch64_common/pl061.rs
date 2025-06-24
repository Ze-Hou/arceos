use tock_registers::{registers::{ReadWrite, ReadOnly, WriteOnly}, register_structs, register_bitfields};

register_bitfields![
    u32,
    pub GPIOIE [
        Pin0 OFFSET(0) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin1 OFFSET(1) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin2 OFFSET(2) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin3 OFFSET(3) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin4 OFFSET(4) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin5 OFFSET(5) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin6 OFFSET(6) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin7 OFFSET(7) NUMBITS(1) [
            clear = 0,
            set = 1
        ]
    ],
    pub GPIOIC [
        Pin0 OFFSET(0) NUMBITS(1) [
            set = 1
        ],
        Pin1 OFFSET(1) NUMBITS(1) [
            set = 1
        ],
        Pin2 OFFSET(2) NUMBITS(1) [
            set = 1
        ],
        Pin3 OFFSET(3) NUMBITS(1) [
            set = 1
        ],
        Pin4 OFFSET(4) NUMBITS(1) [
            set = 1
        ],
        Pin5 OFFSET(5) NUMBITS(1) [
            set = 1
        ],
        Pin6 OFFSET(6) NUMBITS(1) [
            set = 1
        ],
        Pin7 OFFSET(7) NUMBITS(1) [
            set = 1
        ]
    ]
];

register_structs! {
    pub PL061Regs {
        (0x000 => pub data: ReadWrite<u32>),
        (0x004 => __reserved_0),
        (0x400 => pub dir: ReadWrite<u32>),
        (0x404 => pub is: ReadWrite<u32>),
        (0x408 => pub ibe: ReadWrite<u32>),
        (0x40C => pub iev: ReadWrite<u32>),
        (0x410 => pub ie: ReadWrite<u32, GPIOIE::Register>),
        (0x414 => pub ris: ReadOnly<u32>),
        (0x418 => pub mis: ReadOnly<u32>),
        (0x41C => pub ic: WriteOnly<u32, GPIOIC::Register>),
        (0x420 => pub afsel: ReadWrite<u32>),
        (0x424 => @END),
    }
}


use memory_addr::PhysAddr;
use crate::mem::phys_to_virt;
const GPIO_IRQ_NUM: usize = crate::platform::irq::GPIO_IRQ_NUM;

const GPIO_BASE: PhysAddr = pa!(axconfig::devices::GPIO_PADDR);
const GPIO: *mut PL061Regs = (phys_to_virt(GPIO_BASE).as_usize()) as *mut PL061Regs;

pub fn init_gpio() {
    info!("GPIO_IRQ_NUM: {}", GPIO_IRQ_NUM);
    info!("GPIO: {:#x}", GPIO as usize);

    #[cfg(feature = "irq")] {
        use tock_registers::interfaces::{Readable, Writeable};
        crate::irq::set_enable(GPIO_IRQ_NUM, true);
        super::gic::register_handler(GPIO_IRQ_NUM, handle);

        unsafe {
            let p1061r: &PL061Regs = &*GPIO;
            info!("GPIORIS value: {:#x}", p1061r.ris.get());
            info!("GPIOIE value: {:#x}", p1061r.ie.get());
            p1061r.ie.write(GPIOIE::Pin3::set);
            info!("GPIOIE value: {:#x}", p1061r.ie.get());
        }
    }
}

pub fn handle() {
    use core::arch::asm;
    use tock_registers::interfaces::{Readable, Writeable};

    info!("Power Off by GPIO");

    unsafe {
        let p1061r: &PL061Regs = &*GPIO;

        // 清除中断信号 此时get到的应该是0x8
        // .set(); 设置原始寄存器值；.get(); 获取原始寄存器值
        info!("GPIORIS value: {:#x}", p1061r.ris.get());
        p1061r.ic.set(p1061r.ie.get());
        info!("GPIORIS value: {:#x}", p1061r.ris.get());
        // 打印关机信息
        info!("Powering off the system...");
        // 关闭
        asm!("mov w0, #0x18");
        asm!("hlt #0xF000");
    }
}