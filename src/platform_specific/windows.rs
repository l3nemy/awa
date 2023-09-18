#![allow(dead_code)]

use winapi::um::winuser::{GetDesktopWindow, SetParent};
use winit::platform::windows::WindowExtWindows;

pub(crate) fn set_desktop_window(window: &Window) {
    unsafe {
        let desktop_window = GetDesktopWindow();
        SetParent(window.hwnd() as _, desktop_window);
    }
}
