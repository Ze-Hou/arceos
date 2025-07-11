#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

use core::time;
use axhal::console::UART2;
use axhal::console::uart2_init;

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Hello, world!");
    println!("start looping...");
    
    uart2_init();
    UART2.lock().disable();
    UART2.lock().set_baudrate(460800);
    UART2.lock().enable();

    let mut data: u8 = 0;
    loop {
        UART2.lock().write_byte_poll(data);
        let received_data = UART2.lock().read_byte_poll();
        println!("UART2 Demo: Sent {}, Received {}", data, received_data);
        data += 1;
        // 延时1s
        axstd::thread::sleep(time::Duration::from_secs(1));
    }
}
