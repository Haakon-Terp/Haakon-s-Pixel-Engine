//! X11 backend: raw Xlib FFI, no crates. Works on Wayland through XWayland.
//! Assumes the default visual is 24/32-bit TrueColor with 0x00RRGGBB layout
//! (true on practically every modern X server).

use crate::input::{Event, Key, MouseButton, DIGITS, FKEYS, LETTERS};
use crate::Error;
use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void, CString};
use std::ptr;

type Display = c_void;
type XWin = c_ulong;

#[repr(C)]
struct XImage {
    width: c_int,
    height: c_int,
    xoffset: c_int,
    format: c_int,
    data: *mut c_char,
    // remaining fields unused (opaque)
}

#[repr(C)]
#[derive(Clone, Copy)]
struct XKeyEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    window: XWin,
    root: XWin,
    subwindow: XWin,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    keycode: c_uint,
    same_screen: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct XButtonEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    window: XWin,
    root: XWin,
    subwindow: XWin,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    button: c_uint,
    same_screen: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct XMotionEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    window: XWin,
    root: XWin,
    subwindow: XWin,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    is_hint: c_char,
    same_screen: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct XConfigureEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    event: XWin,
    window: XWin,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    above: XWin,
    override_redirect: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct XClientMessageEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    window: XWin,
    message_type: c_ulong,
    format: c_int,
    l: [c_long; 5],
}
#[repr(C)]
union XEvent {
    type_: c_int,
    key: XKeyEvent,
    button: XButtonEvent,
    motion: XMotionEvent,
    configure: XConfigureEvent,
    client: XClientMessageEvent,
    pad: [c_long; 24],
}
#[repr(C)]
struct XSizeHints {
    flags: c_long,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    min_width: c_int,
    min_height: c_int,
    max_width: c_int,
    max_height: c_int,
    width_inc: c_int,
    height_inc: c_int,
    min_aspect: [c_int; 2],
    max_aspect: [c_int; 2],
    base_width: c_int,
    base_height: c_int,
    win_gravity: c_int,
}

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(name: *const c_char) -> *mut Display;
    fn XCloseDisplay(d: *mut Display) -> c_int;
    fn XDefaultScreen(d: *mut Display) -> c_int;
    fn XRootWindow(d: *mut Display, screen: c_int) -> XWin;
    fn XCreateSimpleWindow(
        d: *mut Display, parent: XWin, x: c_int, y: c_int, w: c_uint, h: c_uint,
        border_w: c_uint, border: c_ulong, bg: c_ulong,
    ) -> XWin;
    fn XDestroyWindow(d: *mut Display, w: XWin) -> c_int;
    fn XMapWindow(d: *mut Display, w: XWin) -> c_int;
    fn XStoreName(d: *mut Display, w: XWin, name: *const c_char) -> c_int;
    fn XSelectInput(d: *mut Display, w: XWin, mask: c_long) -> c_int;
    fn XInternAtom(d: *mut Display, name: *const c_char, only_if_exists: c_int) -> c_ulong;
    fn XSetWMProtocols(d: *mut Display, w: XWin, protocols: *mut c_ulong, count: c_int) -> c_int;
    fn XSetWMNormalHints(d: *mut Display, w: XWin, hints: *mut XSizeHints);
    fn XPending(d: *mut Display) -> c_int;
    fn XNextEvent(d: *mut Display, ev: *mut XEvent) -> c_int;
    fn XCreateGC(d: *mut Display, drawable: c_ulong, mask: c_ulong, values: *mut c_void) -> *mut c_void;
    fn XFreeGC(d: *mut Display, gc: *mut c_void) -> c_int;
    fn XDefaultVisual(d: *mut Display, screen: c_int) -> *mut c_void;
    fn XDefaultDepth(d: *mut Display, screen: c_int) -> c_int;
    fn XCreateImage(
        d: *mut Display, visual: *mut c_void, depth: c_uint, format: c_int, offset: c_int,
        data: *mut c_char, w: c_uint, h: c_uint, pad: c_int, bytes_per_line: c_int,
    ) -> *mut XImage;
    fn XPutImage(
        d: *mut Display, drawable: c_ulong, gc: *mut c_void, img: *mut XImage,
        sx: c_int, sy: c_int, dx: c_int, dy: c_int, w: c_uint, h: c_uint,
    ) -> c_int;
    fn XFlush(d: *mut Display) -> c_int;
    fn XFree(p: *mut c_void) -> c_int;
    fn XLookupKeysym(ev: *mut XKeyEvent, index: c_int) -> c_ulong;
    fn XLookupString(
        ev: *mut XKeyEvent, buf: *mut c_char, n: c_int, keysym: *mut c_ulong, status: *mut c_void,
    ) -> c_int;
    fn XkbSetDetectableAutoRepeat(d: *mut Display, detectable: c_int, supported: *mut c_int) -> c_int;
}

const KEY_PRESS: c_int = 2;
const KEY_RELEASE: c_int = 3;
const BUTTON_PRESS: c_int = 4;
const BUTTON_RELEASE: c_int = 5;
const MOTION_NOTIFY: c_int = 6;
const FOCUS_IN: c_int = 9;
const FOCUS_OUT: c_int = 10;
const CONFIGURE_NOTIFY: c_int = 22;
const CLIENT_MESSAGE: c_int = 33;

const KEY_PRESS_MASK: c_long = 1 << 0;
const KEY_RELEASE_MASK: c_long = 1 << 1;
const BUTTON_PRESS_MASK: c_long = 1 << 2;
const BUTTON_RELEASE_MASK: c_long = 1 << 3;
const POINTER_MOTION_MASK: c_long = 1 << 6;
const BUTTON_MOTION_MASK: c_long = 1 << 13;
const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;
const FOCUS_CHANGE_MASK: c_long = 1 << 21;

const Z_PIXMAP: c_int = 2;
const P_MIN_SIZE: c_long = 1 << 4;
const P_MAX_SIZE: c_long = 1 << 5;

fn map_keysym(ks: c_ulong) -> Option<Key> {
    Some(match ks {
        0x61..=0x7A => LETTERS[(ks - 0x61) as usize],
        0x41..=0x5A => LETTERS[(ks - 0x41) as usize],
        0x30..=0x39 => DIGITS[(ks - 0x30) as usize],
        0xFFBE..=0xFFC9 => FKEYS[(ks - 0xFFBE) as usize],
        0x20 => Key::Space,
        0xFF1B => Key::Escape,
        0xFF0D | 0xFF8D => Key::Enter,
        0xFF09 => Key::Tab,
        0xFF08 => Key::Backspace,
        0xFFFF => Key::Delete,
        0xFF63 => Key::Insert,
        0xFF50 => Key::Home,
        0xFF57 => Key::End,
        0xFF55 => Key::PageUp,
        0xFF56 => Key::PageDown,
        0xFF51 => Key::Left,
        0xFF52 => Key::Up,
        0xFF53 => Key::Right,
        0xFF54 => Key::Down,
        0xFFE1 | 0xFFE2 => Key::Shift,
        0xFFE3 | 0xFFE4 => Key::Ctrl,
        0xFFE9 | 0xFFEA | 0xFE03 => Key::Alt,
        _ => return None,
    })
}

pub struct Platform {
    dpy: *mut Display,
    win: XWin,
    gc: *mut c_void,
    visual: *mut c_void,
    depth: c_int,
    wm_delete: c_ulong,
    size: (u32, u32),
    scratch: Vec<u32>,
}

impl Platform {
    pub fn new(title: &str, w: u32, h: u32, resizable: bool) -> Result<Self, Error> {
        unsafe {
            let dpy = XOpenDisplay(ptr::null());
            if dpy.is_null() {
                return Err(Error("cannot open X display (is DISPLAY set?)".into()));
            }
            let screen = XDefaultScreen(dpy);
            let root = XRootWindow(dpy, screen);
            let win = XCreateSimpleWindow(dpy, root, 0, 0, w, h, 0, 0, 0);
            if win == 0 {
                XCloseDisplay(dpy);
                return Err(Error("XCreateSimpleWindow failed".into()));
            }
            XSelectInput(
                dpy,
                win,
                KEY_PRESS_MASK | KEY_RELEASE_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK
                    | POINTER_MOTION_MASK | BUTTON_MOTION_MASK | STRUCTURE_NOTIFY_MASK
                    | FOCUS_CHANGE_MASK,
            );
            XkbSetDetectableAutoRepeat(dpy, 1, ptr::null_mut());

            let ctitle = CString::new(title.replace('\0', "")).unwrap();
            XStoreName(dpy, win, ctitle.as_ptr());

            let mut wm_delete = XInternAtom(dpy, b"WM_DELETE_WINDOW\0".as_ptr() as *const c_char, 0);
            XSetWMProtocols(dpy, win, &mut wm_delete, 1);

            if !resizable {
                let mut hints: XSizeHints = std::mem::zeroed();
                hints.flags = P_MIN_SIZE | P_MAX_SIZE;
                hints.min_width = w as c_int;
                hints.max_width = w as c_int;
                hints.min_height = h as c_int;
                hints.max_height = h as c_int;
                XSetWMNormalHints(dpy, win, &mut hints);
            }

            let gc = XCreateGC(dpy, win, 0, ptr::null_mut());
            let visual = XDefaultVisual(dpy, screen);
            let depth = XDefaultDepth(dpy, screen);
            XMapWindow(dpy, win);
            XFlush(dpy);

            Ok(Platform { dpy, win, gc, visual, depth, wm_delete, size: (w, h), scratch: Vec::new() })
        }
    }

    pub fn poll(&mut self, out: &mut Vec<Event>) {
        unsafe {
            while XPending(self.dpy) > 0 {
                let mut ev: XEvent = std::mem::zeroed();
                XNextEvent(self.dpy, &mut ev);
                match ev.type_ {
                    KEY_PRESS => {
                        let mut k = ev.key;
                        let ks = XLookupKeysym(&mut k, 0);
                        if let Some(key) = map_keysym(ks) {
                            out.push(Event::KeyDown(key));
                        }
                        let mut buf = [0 as c_char; 16];
                        let n = XLookupString(&mut k, buf.as_mut_ptr(), 16, ptr::null_mut(), ptr::null_mut());
                        for i in 0..n.max(0) as usize {
                            let b = buf[i] as u8;
                            // Latin-1 only (no input-method support); ASCII is always right.
                            if b >= 0x20 && b != 0x7F {
                                out.push(Event::Char(b as char));
                            }
                        }
                    }
                    KEY_RELEASE => {
                        let mut k = ev.key;
                        let ks = XLookupKeysym(&mut k, 0);
                        if let Some(key) = map_keysym(ks) {
                            out.push(Event::KeyUp(key));
                        }
                    }
                    BUTTON_PRESS | BUTTON_RELEASE => {
                        let b = ev.button;
                        let down = b.type_ == BUTTON_PRESS;
                        match b.button {
                            1 => out.push(if down { Event::MouseDown(MouseButton::Left) } else { Event::MouseUp(MouseButton::Left) }),
                            2 => out.push(if down { Event::MouseDown(MouseButton::Middle) } else { Event::MouseUp(MouseButton::Middle) }),
                            3 => out.push(if down { Event::MouseDown(MouseButton::Right) } else { Event::MouseUp(MouseButton::Right) }),
                            4 if down => out.push(Event::Scroll(1.0)),
                            5 if down => out.push(Event::Scroll(-1.0)),
                            _ => {}
                        }
                        out.push(Event::MouseMove(b.x, b.y));
                    }
                    MOTION_NOTIFY => {
                        let m = ev.motion;
                        out.push(Event::MouseMove(m.x, m.y));
                    }
                    FOCUS_IN => out.push(Event::Focus(true)),
                    FOCUS_OUT => out.push(Event::Focus(false)),
                    CONFIGURE_NOTIFY => {
                        let c = ev.configure;
                        if c.width > 0 && c.height > 0 {
                            self.size = (c.width as u32, c.height as u32);
                        }
                    }
                    CLIENT_MESSAGE => {
                        if ev.client.l[0] as c_ulong == self.wm_delete {
                            out.push(Event::Close);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn client_size(&self) -> (u32, u32) {
        self.size
    }

    pub fn present(&mut self, pixels: &[u32], w: u32, h: u32) {
        let (cw, ch) = self.size;
        if cw == 0 || ch == 0 {
            return;
        }
        let data: *const u32 = if (cw, ch) == (w, h) {
            pixels.as_ptr()
        } else {
            // nearest-neighbour stretch
            self.scratch.resize(cw as usize * ch as usize, 0);
            let (w, h, cw, ch) = (w as usize, h as usize, cw as usize, ch as usize);
            for dy in 0..ch {
                let src_row = &pixels[(dy * h / ch) * w..][..w];
                let dst_row = &mut self.scratch[dy * cw..][..cw];
                for (dx, d) in dst_row.iter_mut().enumerate() {
                    *d = src_row[dx * w / cw];
                }
            }
            self.scratch.as_ptr()
        };
        unsafe {
            let img = XCreateImage(
                self.dpy, self.visual, self.depth as c_uint, Z_PIXMAP, 0,
                data as *mut c_char, cw, ch, 32, 0,
            );
            if img.is_null() {
                return;
            }
            XPutImage(self.dpy, self.win, self.gc, img, 0, 0, 0, 0, cw, ch);
            (*img).data = ptr::null_mut(); // we own the pixel memory
            XFree(img as *mut c_void);
            XFlush(self.dpy);
        }
    }

    pub fn set_title(&mut self, title: &str) {
        if let Ok(t) = CString::new(title.replace('\0', "")) {
            unsafe {
                XStoreName(self.dpy, self.win, t.as_ptr());
                XFlush(self.dpy);
            }
        }
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        unsafe {
            XFreeGC(self.dpy, self.gc);
            XDestroyWindow(self.dpy, self.win);
            XCloseDisplay(self.dpy);
        }
    }
}