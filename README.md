# pixelwin

A tiny, dependency-free window library for Rust. It does exactly this:

- open a window (optionally resizable, integer-scaled)
- a **double-buffered** 32-bit pixel buffer (`set_pixel`, `get_pixel`, `clear`, raw `pixels_mut`)
- **colour** (`Color::rgb`, `hex`, `from_hsv`, `lerp`, constants)
- **input**: keyboard (down/pressed/released), mouse position/buttons/wheel, typed characters, focus
- frame limiter + delta time

No lines, circles, text or any other drawing. Build those on top of `set_pixel`.

Backends: Windows (Win32/GDI) and Linux/BSD (Xlib, also via XWayland). macOS is not supported.

```rust
use pixelwin::{Window, Config, Color, Key};

let mut win = Window::new("hi", 320, 180, Config { scale: 3, ..Default::default() })?;
while win.is_open() {
    win.poll_events();
    if win.key_pressed(Key::Escape) { win.close(); }
    win.clear(Color::BLACK);
    win.set_pixel(10, 10, Color::rgb(255, 128, 0));
    win.present();   // swaps buffers and shows the frame
}
```

Run the demo: `cargo run --release --example demo`
(Linux needs the libX11 dev package, e.g. `libx11-dev`.)

Note: after `present()` the back buffer holds the frame from two presents ago, so redraw or `clear` every frame.