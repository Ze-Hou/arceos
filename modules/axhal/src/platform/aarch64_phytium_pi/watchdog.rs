use aarch64_cpu::registers::{Readable, Writeable};
use core::ptr::NonNull;
use tock_registers::{register_structs, registers::ReadWrite};

/* 
 * Watchdog 寄存器定义
 * wrr: 看门狗更新寄存器
 * w_iidr: 看门狗接口身份识别寄存器
 * wcs: 看门狗控制和状态寄存器
 * wor: 看门狗清除寄存器
 * wcvl: 看门狗比较值低32位寄存器
 * wcvh: 看门狗比较值高32位寄存器
 */
register_structs! {
    pub WatchDogRegs {
        (0x0000 => pub wrr: ReadWrite<u32>),
        (0x0004 => __reserved0),
        (0x0fcc => pub w_iidr: ReadWrite<u32>),
        (0x0fd0 => __reserved1),
        (0x1000 => pub wcs: ReadWrite<u32>),
        (0x1004 => __reserved2),
        (0x1008 => pub wor: ReadWrite<u32>),
        (0x100c => __reserved3),
        (0x1010 => pub wcvl: ReadWrite<u32>),
        (0x1014 => pub wcvh: ReadWrite<u32>),
        (0x1018 => @END),
    }
}

pub struct WatchDog {
    base: NonNull<WatchDogRegs>,
}


unsafe impl Send for WatchDog {}
unsafe impl Sync for WatchDog {}


impl WatchDog {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &WatchDogRegs {
        unsafe { self.base.as_ref() }
    }

    // 启动看门狗
    pub fn start(&mut self) {
        self.regs().wcs.set(0x1);
    }

    // 停止看门狗
    pub fn stop(&mut self) {
        self.regs().wcs.set(0x0);
    }

    pub fn init(&mut self) {
        self.start();
        // 是能后打印看门狗信息
        let mut temp: u32 = self.regs().w_iidr.get();
        info!("watchdog version: {}, continuation_code: {}, identity_code: {}",
            (temp >> 16) as u8,      // 版本号
            (temp >> 8 & 0xFF) as u8, // 续码
            (temp & 0xFF) as u8       // 身份码
        );
        temp = self.regs().wcs.get();
        if temp & 0x1 != 0 {
            info!("WatchDog is initialized");
            info!("ws1: {}, ws0: {}",
                (temp >> 2 & 0x1) as u8, // ws1
                (temp >> 1 & 0x1) as u8, // ws0
            );
        } 
        else {
            warn!("WatchDog initialization failed");
        }
    }

    // 需要改变计数超时值，可以直接写WOR寄存器。如果WOR的32位寄存器不够计数需求，可以直接写64位的WCV寄存器。
    pub fn set_wor(&mut self, timeout: u32) {
        self.regs().wor.set(timeout);
    }

    // 获取看门狗超时值
    pub fn get_wor(&self) -> u32 {
        
        self.regs().wor.get()
    }

    pub fn set_wcv(&mut self, wcv: u64) {
        // 必须先写高位
        self.regs().wcvh.set((wcv >> 32) as u32);
        self.regs().wcvl.set(wcv as u32);
    }

    // 获取看门狗比较值
    pub fn get_wcv(&self) -> u64 {
        
        let wcvh = self.regs().wcvh.get() as u64;
        let wcvl = self.regs().wcvl.get() as u64;
        (wcvh << 32) | wcvl
    }

    // 喂狗操作--写WRR寄存器
    pub fn feed(&mut self) {
        self.regs().wrr.set(0x1);
    }
}

use kspin::SpinNoIrq;
use memory_addr::PhysAddr;
use crate::mem::phys_to_virt;
use crate::platform::aarch64_common::gic::WATCHDOG_IRQ_NUM;
use aarch64_cpu::asm::nop;

const WDT_BASE: PhysAddr = pa!(0x2804_0000); // WatchDog base address

static WDT: SpinNoIrq<WatchDog> =
    SpinNoIrq::new(WatchDog::new(phys_to_virt(WDT_BASE).as_mut_ptr()));

pub fn Watchdog_init() {
    crate::irq::set_enable(WATCHDOG_IRQ_NUM, true);
    crate::platform::aarch64_common::gic::register_handler(WATCHDOG_IRQ_NUM, handle_wdt_irq);

    WDT.lock().init(); // 初始化看门狗
    // WDT.lock().set_wor(0xF000000); // 设置看门狗超时值为0xF000000
    let wor_value = WDT.lock().get_wor();
    let wcv_value = WDT.lock().get_wcv();
    info!("WatchDog initialized with WOR: {:#X}, WCV: {:#X}", wor_value, wcv_value);

    // 无法触发中断，循环喂狗测试
    for i in 0..10 {
        WDT.lock().feed();
        info!("Feeding WatchDog, iteration: {}", i);
        nop(); // 插入空周期
    }

}

fn handle_wdt_irq() {
    debug!("WatchDog IRQ triggered");
    WDT.lock().feed();
}