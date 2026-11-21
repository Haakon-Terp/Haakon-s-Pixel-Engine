#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Left, Right, Up, Down,
    Escape, Enter, Tab, Space, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
    Shift, Ctrl, Alt,
}

impl Key {
    pub const COUNT: usize = Key::Alt as usize + 1;
}

pub(crate) const LETTERS: [Key; 26] = [
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K,
    Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V,
    Key::W, Key::X, Key::Y, Key::Z,
];
pub(crate) const DIGITS: [Key; 10] = [
    Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4,
    Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9,
];
pub(crate) const FKEYS: [Key; 12] = [
    Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
    Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

/// Raw events produced by the platform backends.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
    Close,
    Focus(bool),
    KeyDown(Key),
    KeyUp(Key),
    Char(char),
    /// Position in window-client pixels (may be outside the window while dragging).
    MouseMove(i32, i32),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    Scroll(f32),
}

/// Per-frame input state. Edge queries (`*_pressed`, `*_released`) are relative
/// to the previous call of `Window::poll_events`.
pub struct Input {
    keys: [bool; Key::COUNT],
    prev_keys: [bool; Key::COUNT],
    buttons: [bool; 3],
    prev_buttons: [bool; 3],
    mouse: (i32, i32),
    prev_mouse: (i32, i32),
    scroll: f32,
    chars: Vec<char>,
    focused: bool,
}

impl Input {
    pub(crate) fn new() -> Self {
        Input {
            keys: [false; Key::COUNT],
            prev_keys: [false; Key::COUNT],
            buttons: [false; 3],
            prev_buttons: [false; 3],
            mouse: (0, 0),
            prev_mouse: (0, 0),
            scroll: 0.0,
            chars: Vec::new(),
            focused: true,
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.prev_keys = self.keys;
        self.prev_buttons = self.buttons;
        self.prev_mouse = self.mouse;
        self.scroll = 0.0;
        self.chars.clear();
    }

    pub(crate) fn key_event(&mut self, k: Key, down: bool) {
        self.keys[k as usize] = down;
    }
    pub(crate) fn button_event(&mut self, b: MouseButton, down: bool) {
        self.buttons[b as usize] = down;
    }
    pub(crate) fn set_mouse(&mut self, p: (i32, i32)) {
        self.mouse = p;
    }
    pub(crate) fn add_scroll(&mut self, s: f32) {
        self.scroll += s;
    }
    pub(crate) fn add_char(&mut self, c: char) {
        self.chars.push(c);
    }
    pub(crate) fn set_focus(&mut self, f: bool) {
        self.focused = f;
        if !f {
            self.keys = [false; Key::COUNT];
            self.buttons = [false; 3];
        }
    }

    pub fn key_down(&self, k: Key) -> bool {
        self.keys[k as usize]
    }
    pub fn key_pressed(&self, k: Key) -> bool {
        self.keys[k as usize] && !self.prev_keys[k as usize]
    }
    pub fn key_released(&self, k: Key) -> bool {
        !self.keys[k as usize] && self.prev_keys[k as usize]
    }

    pub fn mouse_down(&self, b: MouseButton) -> bool {
        self.buttons[b as usize]
    }
    pub fn mouse_pressed(&self, b: MouseButton) -> bool {
        self.buttons[b as usize] && !self.prev_buttons[b as usize]
    }
    pub fn mouse_released(&self, b: MouseButton) -> bool {
        !self.buttons[b as usize] && self.prev_buttons[b as usize]
    }

    /// Mouse position in *buffer* pixels (scaling already undone).
    /// Can be negative / past the edge while a button is held and dragged outside.
    pub fn mouse_pos(&self) -> (i32, i32) {
        self.mouse
    }
    /// Movement since the previous frame, in buffer pixels.
    pub fn mouse_delta(&self) -> (i32, i32) {
        (self.mouse.0 - self.prev_mouse.0, self.mouse.1 - self.prev_mouse.1)
    }
    /// Wheel notches this frame (positive = away from user).
    pub fn scroll(&self) -> f32 {
        self.scroll
    }
    /// Text typed this frame (respects keyboard layout; includes key repeat).
    pub fn typed_chars(&self) -> &[char] {
        &self.chars
    }
    pub fn has_focus(&self) -> bool {
        self.focused
    }
}