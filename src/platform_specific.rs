#![allow(unused_imports)]
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "macos")] {
        mod macos;
        pub(crate) use macos::*;
    }
    else if #[cfg(target_os = "linux")] {
        mod linux;
        pub(crate) use linux::*;

    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub(crate) use windows::*;

    } else {
        compile_error!("Unsupported platform!");
    }
}
