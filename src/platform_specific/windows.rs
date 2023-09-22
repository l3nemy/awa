#![allow(dead_code)]

use std::mem;

use winapi::{
    shared::windef::HWND,
    um::winuser::{
        CloseWindow, EnumWindows, FindWindowExA, FindWindowExW, GetDesktopWindow,
        SendMessageTimeoutW, SetParent, SMTO_NORMAL,
    },
};
use winit::{platform::windows::WindowExtWindows, window::Window};

#[no_mangle]
unsafe extern "system" fn enum_window(hwnd: HWND, lparam: isize) -> i32 {
    let workerw = lparam as *mut HWND;

    let p = FindWindowExA(hwnd, 0 as _, "SHELLDLL_DefView\0".as_ptr() as _, 0 as _);
    if p != 0 as _ {
        *workerw = FindWindowExA(0 as _, hwnd, "WorkerW\0".as_ptr() as _, 0 as _);
    } 
    1
}

// TODO(l3nemy): Make it return Result<(), Error>
pub(crate) fn set_desktop_window(window: &Window) {
    unsafe {
        let window_hwnd = window.hwnd() as HWND;

        let progman = FindWindowExA(GetDesktopWindow(), 0 as _, "Progman\0".as_ptr() as _, "Program Manager\0".as_ptr() as _);

        let mut result: usize = mem::zeroed();
        let result_ptr: *mut usize = &mut result;

        if SendMessageTimeoutW(progman, 0x052C, 0 as _, 0 as _, SMTO_NORMAL, 10000, result_ptr) == 0 {
            panic!("Error creating WorkerW: {:#?}", result);
        }

        let mut workerw: HWND = mem::zeroed();
        let workerw_ptr: *mut HWND = &mut workerw;

        if EnumWindows(Some(enum_window), workerw_ptr as _) == 0 {
            panic!("Error getting WorkerW");
        }

        if workerw as usize == 0 {
            panic!("Error getting WorkerW");
        }

        SetParent(window_hwnd, workerw);
    }
}
