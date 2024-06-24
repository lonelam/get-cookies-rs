#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "windows")]
pub use win::read_cookie::*;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
pub use mac::read_cookie_until::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::read_cookie_until::*;
