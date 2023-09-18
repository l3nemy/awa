#![allow(dead_code)]

use objc::{
    class, msg_send,
    runtime::{Object, NO},
    sel, sel_impl,
};
use winit::platform::macos::WindowExtMacOS;
use winit::window::Window;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGWindowLevelForKey(key: i32) -> i32;
}

fn set_collection_behavior(window: *mut Object, visible: bool, flag: u64) {
    use std::os::raw::c_ulong;

    unsafe {
        let collection_behavior: c_ulong = msg_send![window, collectionBehavior];

        let flags: c_ulong = if visible {
            collection_behavior | flag
        } else {
            collection_behavior & (!flag)
        };
        let _: () = msg_send![window, setCollectionBehavior: flags];
    }
}

pub(crate) fn set_desktop_window(window: &Window) {
    unsafe {
        let obj: *mut Object = window.ns_window() as *mut Object;

        // 2 : kCGDesktopWindowLevel
        let _: () = msg_send![obj, setLevel: CGWindowLevelForKey(2) - 1];
        let _: () = msg_send![obj, setOpaque: NO];

        let color: *mut Object = msg_send![class!(NSColor), clearColor];
        let _: () = msg_send![obj, setBackgroundColor: color];

        // Make the window fixed on all workspaces
        // Look at: https://github.com/electron/electron/blob/f6e8a42c48b16f04675a507f2bee26020466c380/shell/browser/native_window_mac.mm#L1387
        //          https://github.com/rust-windowing/winit/issues/2151
        //
        // 1 << 0 : NSWindowCollectionBehaviorCanJoinAllSpaces
        // 1 << 8 : NSWindowCollectionBehaviorFullScreenAuxiliary
        set_collection_behavior(obj, true, 1 << 0);
        set_collection_behavior(obj, true, 1 << 8);
    }
}
