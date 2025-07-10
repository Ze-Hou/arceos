#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

use axstd::thread;
use core::time::Duration;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Hello, world!");
    println!("start looping...");
    
    // 定义一个变量，每次循环自加1
    let mut count = 0;
    loop {
        println!("running, count: {}", count);
        thread::sleep(Duration::from_secs(5));
        count += 1;
    }
}
