use crate::Color;

/// Two same-sized pixel buffers. You draw into `back`; `swap` makes it the
/// `front` (what gets shown) and hands you the old front as the new back.
pub struct DoubleBuffer {
    width: usize,
    height: usize,
    front: Vec<u32>,
    back: Vec<u32>,
}

impl DoubleBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        DoubleBuffer {
            width,
            height,
            front: vec![0; width * height],
            back: vec![0; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn front(&self) -> &[u32] {
        &self.front
    }
    pub fn back(&self) -> &[u32] {
        &self.back
    }
    pub fn back_mut(&mut self) -> &mut [u32] {
        &mut self.back
    }

    /// Out-of-bounds writes are ignored.
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Color) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.back[y as usize * self.width + x as usize] = c.0;
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Option<Color> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(Color(self.back[y as usize * self.width + x as usize]))
        } else {
            None
        }
    }

    pub fn clear(&mut self, c: Color) {
        self.back.fill(c.0);
    }
}