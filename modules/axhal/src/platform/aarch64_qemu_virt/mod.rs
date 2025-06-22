pub mod mem;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use crate::platform::aarch64_common::gic::*;
}

pub mod console {
    pub use crate::platform::aarch64_common::pl011::*;
}

pub mod time {
    pub use crate::platform::aarch64_common::generic_timer::*;
}

pub mod misc {
    pub use crate::platform::aarch64_common::psci::system_off as terminate;
}

unsafe extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    axcpu::init::init_trap();
    crate::cpu::init_primary(cpu_id);
    super::aarch64_common::pl011::init_early();
    super::aarch64_common::generic_timer::init_early();
    rust_main(cpu_id, dtb);
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    axcpu::init::init_trap();
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_primary();
    super::aarch64_common::generic_timer::init_percpu();
    super::aarch64_common::pl011::init();
    power_gpio_init();
}

use memory_addr::PhysAddr;
use arm_gicv2::{translate_irq, InterruptType};
use crate::mem::phys_to_virt;
use crate::platform::pl061::PL061Regs;
const PL061REGS_PADDR: PhysAddr = pa!(axconfig::devices::PL061_PADDR);
const PL061REGS: *mut PL061Regs = (phys_to_virt(PL061REGS_PADDR).as_usize()) as *mut PL061Regs;
const GPIO_IRQ_NUM: usize = translate_irq(axconfig::devices::GPIO_IRQ, InterruptType::SPI).unwrap();

// 初始化GPIO中断
pub fn power_gpio_init() {
    use tock_registers::interfaces::{Readable, Writeable};
    use crate::platform::aarch64_common::gic;
    use super::pl061::*;

    // 打印GPIO_IRQ_NUM的值
    info!("GPIO_IRQ: {}", GPIO_IRQ_NUM);
    // //使能中断
    crate::irq::set_enable(GPIO_IRQ_NUM, true);

    // //注册中断处理函数
    gic::register_handler(GPIO_IRQ_NUM, handle_gpio_irq);

    // 打印PL061REGS的值
    info!("PL061REGS: {:#x}", PL061REGS as usize);
    info!("Init GPIO Pin3");
    // 使能GPIO的Poweroff key中断
    unsafe {
        let p1061r: &PL061Regs = &*PL061REGS;

        info!("GPIORIS value: {:#x}", p1061r.ris.get());
        info!("GPIOIE value: {:#x}", p1061r.ie.get());
        p1061r.ie.write(GPIOIE::Pin3::set);
        info!("GPIOIE value: {:#x}", p1061r.ie.get());
    }
}

pub fn handle_gpio_irq() {
    use core::arch::asm;
    use tock_registers::interfaces::{Readable, Writeable};

    info!("Power Off by GPIO");

    unsafe {
        let p1061r: &PL061Regs = &*PL061REGS;

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

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_secondary();
    super::aarch64_common::generic_timer::init_percpu();
}
