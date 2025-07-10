mod boot;

pub mod generic_timer;
#[cfg(not(platform_family = "aarch64-raspi"))]
pub mod psci;

#[cfg(all(feature = "irq", feature = "gicv3"))]
pub mod gicv3;
#[cfg(all(feature = "irq", feature = "gicv3"))]
pub use gicv3 as gic;

#[cfg(all(feature = "irq", not(feature = "gicv3")))]
pub mod gic;

#[cfg(not(platform_family = "aarch64-bsta1000b"))]
pub mod pl011;
