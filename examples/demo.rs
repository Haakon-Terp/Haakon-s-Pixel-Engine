//! Left mouse: plot pixels. Right mouse: clear. Esc: quit.
//! Only `set_pixel` / raw buffer access is used; there are no draw primitives.

use pixelwin::{Color, Config, Key, MouseButton, Window};
use std::time::Instant;

fn main() {
    let (w, h) = (320usize, 180usize);
    let mut win = Window::new(
        "pixelwin demo",
        w,
        h,
        Config { scale: 4, resizable: true, target_fps: Some(60) },
    )
    .expect("failed to create window");

    let mut canvas = vec![0u32; w * h]; // persistent user drawing
    let start = Instant::now();
    let mut frames = 0u32;
    let mut fps_timer = Instant::now();

    while win.is_open() {
        win.poll_events();
        if win.key_pressed(Key::Escape) {
            win.close();
        }

        // user input -> persistent canvas (a "brush" is just a loop of pixels)
        let (mx, my) = win.input().mouse_pos();
        let hue = (start.elapsed().as_secs_f32() * 60.0) % 360.0;
        if win.input().mouse_down(MouseButton::Left) {
            let c = Color::from_hsv(hue, 1.0, 1.0);
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let (x, y) = (mx + dx, my + dy);
                    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                        canvas[y as usize * w + x as usize] = c.0;
                    }
                }
            }
        }
        if win.input().mouse_pressed(MouseButton::Right) {
            canvas.fill(0);
        }

        // render into the back buffer: animated XOR pattern, canvas on top
        let t = (start.elapsed().as_secs_f32() * 40.0) as usize;
        for y in 0..h {
            for x in 0..w {
                let p = canvas[y * w + x];
                let c = if p != 0 {
                    Color(p)
                } else {
                    let v = (((x + t) ^ (y + t / 2)) & 0xFF) as u8 / 4;
                    Color::rgb(v / 2, v, v.saturating_add(20))
                };
                win.set_pixel(x as i32, y as i32, c);
            }
        }
        // crosshair cursor
        win.set_pixel(mx, my, Color::WHITE);
        win.set_pixel(mx + 1, my, Color::WHITE);
        win.set_pixel(mx - 1, my, Color::WHITE);
        win.set_pixel(mx, my + 1, Color::WHITE);
        win.set_pixel(mx, my - 1, Color::WHITE);

        win.present();

        frames += 1;
        if fps_timer.elapsed().as_secs_f32() >= 1.0 {
            win.set_title(&format!("pixelwin demo | {} fps", frames));
            frames = 0;
            fps_timer = Instant::now();
        }
    }
}