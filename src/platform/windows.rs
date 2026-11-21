//! Win32 backend: user32 + gdi32 via raw FFI, no crates.

use crate::input::{Event, Key, MouseButton, DIGITS, FKEYS, LETTERS};
use crate::Error;
use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::ptr;

type Hwnd = *mut c_void;

#[repr(C)]
struct WndClassW {
    style: u32,
    wnd_proc: unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize,
    cls_extra: i32,
    wnd_extra: i32,
    instance: *mut c_void,
    icon: *mut c_void,
    cursor: *mut c_void,
    background: *mut c_void,
    menu_name: *const u16,
    class_name: *const u16,
}
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
    private: u32,
}
#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_ppm: i32,
    y_ppm: i32,
    clr_used: u32,
    clr_important: u32,
}
#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [u32; 1],
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        ex_style: u32, class: *const u16, title: *const u16, style: u32,
        x: i32, y: i32, w: i32, h: i32,
        parent: Hwnd, menu: *mut c_void, instance: *mut c_void, param: *mut c_void,
    ) -> Hwnd;
    fn DestroyWindow(hwnd: Hwnd) -> i32;
    fn ShowWindow(hwnd: Hwnd, cmd: i32) -> i32;
    fn AdjustWindowRect(rect: *mut Rect, style: u32, menu: i32) -> i32;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    fn PeekMessageW(msg: *mut Msg, hwnd: Hwnd, min: u32, max: u32, remove: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn DefWindowProcW(hwnd: Hwnd, msg: u32, wp: usize, lp: isize) -> isize;
    fn LoadCursorW(instance: *mut c_void, name: *const u16) -> *mut c_void;
    fn GetDC(hwnd: Hwnd) -> *mut c_void;
    fn ReleaseDC(hwnd: Hwnd, hdc: *mut c_void) -> i32;
    fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    fn SetCapture(hwnd: Hwnd) -> Hwnd;
    fn ReleaseCapture() -> i32;
    #[cfg(target_pointer_width = "64")]
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    #[cfg(target_pointer_width = "64")]
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    #[cfg(target_pointer_width = "32")]
    #[link_name = "SetWindowLongW"]
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    #[cfg(target_pointer_width = "32")]
    #[link_name = "GetWindowLongW"]
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
}
#[link(name = "gdi32")]
extern "system" {
    fn SetStretchBltMode(hdc: *mut c_void, mode: i32) -> i32;
    fn StretchDIBits(
        hdc: *mut c_void, xd: i32, yd: i32, wd: i32, hd: i32,
        xs: i32, ys: i32, ws: i32, hs: i32,
        bits: *const c_void, info: *const BitmapInfo, usage: u32, rop: u32,
    ) -> i32;
}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

const GWLP_USERDATA: i32 = -21;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
const WS_THICKFRAME: u32 = 0x0004_0000;
const WS_MAXIMIZEBOX: u32 = 0x0001_0000;
const CW_USEDEFAULT: i32 = i32::MIN;
const SW_SHOW: i32 = 5;
const PM_REMOVE: u32 = 1;
const COLORONCOLOR: i32 = 3;
const SRCCOPY: u32 = 0x00CC_0020;
const IDC_ARROW: usize = 32512;

const WM_NCCREATE: u32 = 0x0081;
const WM_SIZE: u32 = 0x0005;
const WM_SETFOCUS: u32 = 0x0007;
const WM_KILLFOCUS: u32 = 0x0008;
const WM_CLOSE: u32 = 0x0010;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_CHAR: u32 = 0x0102;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_MBUTTONDOWN: u32 = 0x0207;
const WM_MBUTTONUP: u32 = 0x0208;
const WM_MOUSEWHEEL: u32 = 0x020A;

struct Shared {
    events: RefCell<Vec<Event>>,
    size: Cell<(u32, u32)>,
    high_surrogate: Cell<u16>,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn map_vk(vk: usize) -> Option<Key> {
    Some(match vk {
        0x41..=0x5A => LETTERS[vk - 0x41],
        0x30..=0x39 => DIGITS[vk - 0x30],
        0x70..=0x7B => FKEYS[vk - 0x70],
        0x25 => Key::Left,
        0x26 => Key::Up,
        0x27 => Key::Right,
        0x28 => Key::Down,
        0x1B => Key::Escape,
        0x0D => Key::Enter,
        0x09 => Key::Tab,
        0x20 => Key::Space,
        0x08 => Key::Backspace,
        0x2E => Key::Delete,
        0x2D => Key::Insert,
        0x24 => Key::Home,
        0x23 => Key::End,
        0x21 => Key::PageUp,
        0x22 => Key::PageDown,
        0x10 => Key::Shift,
        0x11 => Key::Ctrl,
        0x12 => Key::Alt,
        _ => return None,
    })
}

unsafe extern "system" fn wndproc(hwnd: Hwnd, msg: u32, wp: usize, lp: isize) -> isize {
    if msg == WM_NCCREATE {
        // First field of CREATESTRUCTW is lpCreateParams.
        let create_params = *(lp as *const *mut c_void);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_params as isize);
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Shared;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let sh = &*ptr;
    let push = |e: Event| sh.events.borrow_mut().push(e);
    let mouse_xy = || ((lp & 0xFFFF) as u16 as i16 as i32, ((lp >> 16) & 0xFFFF) as u16 as i16 as i32);

    match msg {
        WM_CLOSE => {
            push(Event::Close);
            0
        }
        WM_SIZE => {
            let w = (lp & 0xFFFF) as u32;
            let h = ((lp >> 16) & 0xFFFF) as u32;
            if w > 0 && h > 0 {
                sh.size.set((w, h));
            }
            0
        }
        WM_SETFOCUS => {
            push(Event::Focus(true));
            0
        }
        WM_KILLFOCUS => {
            push(Event::Focus(false));
            0
        }
        WM_ERASEBKGND => 1,
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if let Some(k) = map_vk(wp) {
                push(Event::KeyDown(k));
            }
            if msg == WM_SYSKEYDOWN { DefWindowProcW(hwnd, msg, wp, lp) } else { 0 }
        }
        WM_KEYUP | WM_SYSKEYUP => {
            if let Some(k) = map_vk(wp) {
                push(Event::KeyUp(k));
            }
            if msg == WM_SYSKEYUP { DefWindowProcW(hwnd, msg, wp, lp) } else { 0 }
        }
        WM_CHAR => {
            let u = wp as u16;
            let c = if (0xD800..0xDC00).contains(&u) {
                sh.high_surrogate.set(u);
                None
            } else if (0xDC00..0xE000).contains(&u) {
                let hi = sh.high_surrogate.replace(0);
                char::decode_utf16([hi, u]).next().and_then(|r| r.ok())
            } else {
                char::from_u32(u as u32)
            };
            if let Some(c) = c {
                if c >= ' ' && c != '\u{7f}' {
                    push(Event::Char(c));
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            let (x, y) = mouse_xy();
            push(Event::MouseMove(x, y));
            0
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
            SetCapture(hwnd);
            let b = match msg {
                WM_LBUTTONDOWN => MouseButton::Left,
                WM_RBUTTONDOWN => MouseButton::Right,
                _ => MouseButton::Middle,
            };
            push(Event::MouseDown(b));
            0
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
            ReleaseCapture();
            let b = match msg {
                WM_LBUTTONUP => MouseButton::Left,
                WM_RBUTTONUP => MouseButton::Right,
                _ => MouseButton::Middle,
            };
            push(Event::MouseUp(b));
            0
        }
        WM_MOUSEWHEEL => {
            let delta = ((wp >> 16) & 0xFFFF) as u16 as i16;
            push(Event::Scroll(delta as f32 / 120.0));
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub struct Platform {
    hwnd: Hwnd,
    shared: Box<Shared>,
}

impl Platform {
    pub fn new(title: &str, w: u32, h: u32, resizable: bool) -> Result<Self, Error> {
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let class_name = wide("PixelWinWindow");
            let class = WndClassW {
                style: 0,
                wnd_proc: wndproc,
                cls_extra: 0,
                wnd_extra: 0,
                instance,
                icon: ptr::null_mut(),
                cursor: LoadCursorW(ptr::null_mut(), IDC_ARROW as *const u16),
                background: ptr::null_mut(),
                menu_name: ptr::null(),
                class_name: class_name.as_ptr(),
            };
            RegisterClassW(&class); // fails harmlessly if already registered

            let mut style = WS_OVERLAPPEDWINDOW;
            if !resizable {
                style &= !(WS_THICKFRAME | WS_MAXIMIZEBOX);
            }
            let mut rect = Rect { left: 0, top: 0, right: w as i32, bottom: h as i32 };
            AdjustWindowRect(&mut rect, style, 0);

            let shared = Box::new(Shared {
                events: RefCell::new(Vec::new()),
                size: Cell::new((w, h)),
                high_surrogate: Cell::new(0),
            });
            let title_w = wide(title);
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title_w.as_ptr(),
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                &*shared as *const Shared as *mut c_void,
            );
            if hwnd.is_null() {
                return Err(Error("CreateWindowExW failed".into()));
            }
            ShowWindow(hwnd, SW_SHOW);

            let mut cr = Rect::default();
            GetClientRect(hwnd, &mut cr);
            if cr.right > 0 && cr.bottom > 0 {
                shared.size.set((cr.right as u32, cr.bottom as u32));
            }
            Ok(Platform { hwnd, shared })
        }
    }

    pub fn poll(&mut self, out: &mut Vec<Event>) {
        unsafe {
            let mut msg: Msg = std::mem::zeroed();
            while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        out.extend(self.shared.events.borrow_mut().drain(..));
    }

    pub fn client_size(&self) -> (u32, u32) {
        self.shared.size.get()
    }

    pub fn present(&mut self, pixels: &[u32], w: u32, h: u32) {
        let (cw, ch) = self.shared.size.get();
        if cw == 0 || ch == 0 {
            return;
        }
        unsafe {
            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                    width: w as i32,
                    height: -(h as i32), // top-down
                    planes: 1,
                    bit_count: 32,
                    compression: 0, // BI_RGB
                    size_image: 0,
                    x_ppm: 0,
                    y_ppm: 0,
                    clr_used: 0,
                    clr_important: 0,
                },
                colors: [0],
            };
            let hdc = GetDC(self.hwnd);
            SetStretchBltMode(hdc, COLORONCOLOR);
            StretchDIBits(
                hdc, 0, 0, cw as i32, ch as i32, 0, 0, w as i32, h as i32,
                pixels.as_ptr() as *const c_void, &info, 0, SRCCOPY,
            );
            ReleaseDC(self.hwnd, hdc);
        }
    }

    pub fn set_title(&mut self, title: &str) {
        let t = wide(title);
        unsafe { SetWindowTextW(self.hwnd, t.as_ptr()) };
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            DestroyWindow(self.hwnd);
        }
    }
}