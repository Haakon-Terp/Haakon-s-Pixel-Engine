//! pixelwin: a window, a double-buffered pixel buffer, colours and input.
//! Nothing else. No lines, no shapes, no text. Build those yourself on `set_pixel`.

mod buffer;
mod color;
mod input;

#[cfg(windows)]
#[path = "platform/windows.rs"]
mod platform;

#[cfg(all(unix, not(target_os = "macos")))]
#[path = "platform/x11.rs"]
mod platform;

#[cfg(not(any(windows, all(unix, not(target_os = "macos")))))]
compile_error!("pixelwin supports Windows and Linux/BSD (X11) only");

pub use buffer::DoubleBuffer;
pub use color::Color;
pub use input::{Input, Key, MouseButton};

use input::Event;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Error(pub String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Integer window-size multiplier over the buffer size.
    pub scale: u32,
    /// If true the user may resize the window; the buffer is stretched
    /// (nearest-neighbour) to fit. The buffer size itself never changes.
    pub resizable: bool,
    /// Frame limiter applied in `present`. `None` = run as fast as possible.
    pub target_fps: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Config { scale: 1, resizable: false, target_fps: Some(60) }
    }
}

pub struct Window {
    platform: platform::Platform,
    buf: DoubleBuffer,
    input: Input,
    events: Vec<Event>,
    open: bool,
    target_frame: Option<Duration>,
    last_present: Instant,
    dt: f32,
}

impl Window {
    pub fn new(title: &str, width: usize, height: usize, config: Config) -> Result<Window, Error> {
        if width == 0 || height == 0 {
            return Err(Error("width and height must be non-zero".into()));
        }
        let scale = config.scale.max(1);
        let platform = platform::Platform::new(
            title,
            width as u32 * scale,
            height as u32 * scale,
            config.resizable,
        )?;
        Ok(Window {
            platform,
            buf: DoubleBuffer::new(width, height),
            input: Input::new(),
            events: Vec::new(),
            open: true,
            target_frame: config.target_fps.filter(|&f| f > 0).map(|f| Duration::from_secs_f64(1.0 / f as f64)),
            last_present: Instant::now(),
            dt: 0.0,
        })
    }

    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Ask the window to close (is_open() becomes false).
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Pump OS messages and update `input()`. Call once per frame.
    pub fn poll_events(&mut self) {
        self.input.begin_frame();
        let mut events = std::mem::take(&mut self.events);
        events.clear();
        self.platform.poll(&mut events);
        let (cw, ch) = self.platform.client_size();
        let (bw, bh) = (self.buf.width() as i64, self.buf.height() as i64);

        for ev in events.iter().copied() {
            match ev {
                Event::Close => self.open = false,
                Event::Focus(f) => self.input.set_focus(f),
                Event::KeyDown(k) => self.input.key_event(k, true),
                Event::KeyUp(k) => self.input.key_event(k, false),
                Event::Char(c) => self.input.add_char(c),
                Event::MouseMove(x, y) => {
                    if cw > 0 && ch > 0 {
                        let bx = (x as i64 * bw).div_euclid(cw as i64) as i32;
                        let by = (y as i64 * bh).div_euclid(ch as i64) as i32;
                        self.input.set_mouse((bx, by));
                    }
                }
                Event::MouseDown(b) => self.input.button_event(b, true),
                Event::MouseUp(b) => self.input.button_event(b, false),
                Event::Scroll(s) => self.input.add_scroll(s),
            }
        }
        self.events = events;
    }

    /// Swap the buffers and show the finished frame, then apply the frame limiter.
    ///
    /// After this call the back buffer holds the frame from *two* presents ago,
    /// so redraw / `clear` it fully every frame.
    pub fn present(&mut self) {
        self.buf.swap();
        let (w, h) = (self.buf.width() as u32, self.buf.height() as u32);
        self.platform.present(self.buf.front(), w, h);

        if let Some(target) = self.target_frame {
            let elapsed = self.last_present.elapsed();
            if elapsed < target {
                let remaining = target - elapsed;
                if remaining > Duration::from_millis(2) {
                    std::thread::sleep(remaining - Duration::from_millis(1));
                }
                while self.last_present.elapsed() < target {
                    std::hint::spin_loop();
                }
            }
        }
        let now = Instant::now();
        self.dt = (now - self.last_present).as_secs_f32();
        self.last_present = now;
    }

    // ---- pixel access (always the back buffer) ----

    pub fn width(&self) -> usize {
        self.buf.width()
    }
    pub fn height(&self) -> usize {
        self.buf.height()
    }

    /// Plot one pixel. Out-of-bounds is silently ignored.
    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, c: Color) {
        self.buf.set(x, y, c);
    }
    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        self.buf.get(x, y)
    }
    pub fn clear(&mut self, c: Color) {
        self.buf.clear(c);
    }
    /// Raw back buffer, row-major, `width*height` entries of 0x00RRGGBB.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        self.buf.back_mut()
    }
    pub fn pixels(&self) -> &[u32] {
        self.buf.back()
    }

    // ---- misc ----

    pub fn input(&self) -> &Input {
        &self.input
    }
    pub fn key_down(&self, k: Key) -> bool {
        self.input.key_down(k)
    }
    pub fn key_pressed(&self, k: Key) -> bool {
        self.input.key_pressed(k)
    }
    /// Seconds between the last two `present` calls.
    pub fn delta_time(&self) -> f32 {
        self.dt
    }
    pub fn set_title(&mut self, title: &str) {
        self.platform.set_title(title);
    }
}