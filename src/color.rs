/// 24-bit colour stored as 0x00RRGGBB.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Color(pub u32);

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);
    pub const CYAN: Color = Color::rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::rgb(255, 0, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    /// From 0xRRGGBB (top byte ignored).
    pub const fn hex(v: u32) -> Color {
        Color(v & 0x00FF_FFFF)
    }

    pub const fn r(self) -> u8 {
        (self.0 >> 16) as u8
    }
    pub const fn g(self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub const fn b(self) -> u8 {
        self.0 as u8
    }

    /// Hue in degrees (wraps), saturation and value in 0..=1.
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Color {
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let h = h.rem_euclid(360.0) / 60.0;
        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let f = |n: f32| ((n + m) * 255.0 + 0.5) as u8;
        Color::rgb(f(r), f(g), f(b))
    }

    /// Linear interpolation, t clamped to 0..=1.
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t + 0.5) as u8;
        Color::rgb(m(self.r(), other.r()), m(self.g(), other.g()), m(self.b(), other.b()))
    }
}

impl From<u32> for Color {
    fn from(v: u32) -> Self {
        Color::hex(v)
    }
}
impl From<(u8, u8, u8)> for Color {
    fn from(c: (u8, u8, u8)) -> Self {
        Color::rgb(c.0, c.1, c.2)
    }
}