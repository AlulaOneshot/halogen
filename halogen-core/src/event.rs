use downcast_rs::impl_downcast;
use crate::geometry::Point;

pub enum EventResult {
    Consumed,
    Bubble,
}

// Core Enum

pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Touch(TouchEvent),
    Window(WindowEvent),
    Gamepad(GamepadEvent),
    Custom(Box<dyn CustomEvent>),
}

// Mouse

pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub position: Point<f32>,
    pub modifiers: Modifiers,
}

pub enum MouseEventKind {
    Pressed(MouseButton),
    Released(MouseButton),
    Moved,
    Entered,
    Exited,
    Scrolled(ScrollDelta),
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

pub enum ScrollDelta {
    Lines { x: f32, y: f32 },
    Pixels { x: f32, y: f32 },
}

// Keyboard

pub struct KeyboardEvent {
    pub kind: KeyboardEventKind,
    pub key: Key,
    pub modifiers: Modifiers,
}

pub enum KeyboardEventKind {
    Pressed,
    Released,
    Repeated,
}

pub enum Key {
    // Printable
    Char(char),
    // Control
    Backspace,
    Delete,
    Enter,
    Escape,
    Tab,
    Space,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Function(u8), // F1–F24
    // Platform
    Other(u32), // fallback to raw keycode
}

// Touch

pub struct TouchEvent {
    pub kind: TouchEventKind,
    pub id: u64, // finger ID for multitouch displays
    pub position: Point<f32>,
    pub force: Option<f32>, // [0.0, 1.0] when available
}

pub enum TouchEventKind {
    Began,
    Moved,
    Ended,
    Cancelled,
}

// Window

pub enum WindowEvent {
    Resized { width: f32, height: f32 },
    Focused,
    Unfocused,
    CloseRequested,
    ScaleFactorChanged(f32),
}

// Gamepad

pub struct GamepadEvent {
    pub id: u32,
    pub kind: GamepadEventKind,
}

pub enum GamepadEventKind {
    ButtonPressed(GamepadButton),
    ButtonReleased(GamepadButton),
    AxisChanged(GamepadAxis, f32), // [-1.0, 1.0]
    Connected,
    Disconnected,
}

pub enum GamepadButton {
    South,
    North,
    East,
    West, // face buttons
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Start,
    Select,
    Other(u16),
}

pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

// Shared

#[derive(Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Cmd on Mac, Win on Windows, Super on Linux
}

// User Extension

pub trait CustomEvent: downcast_rs::Downcast + Send + Sync + 'static {}
impl_downcast!(CustomEvent);