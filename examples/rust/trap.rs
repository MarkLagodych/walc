#![no_std]
#![no_main]
mod walc;

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable()
}
