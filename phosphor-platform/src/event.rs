use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::scancode::PhysicalKeyExtScancode;
use phosphor_core::event::{Event, FocusEvent, Key, KeyDownEvent, KeyUpEvent, Modifiers};

pub fn translate_window_event(event: &WindowEvent) -> Option<Event<()>> {
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            let key = translate_key(&event.physical_key);
            let modifiers = Modifiers::NONE; // updated separately via ModifiersChanged
            let text = event.text.as_ref().map(|t| t.to_string());

            match event.state {
                ElementState::Pressed => Some(Event::KeyDown(KeyDownEvent {
                    key,
                    modifiers,
                    text,
                    is_repeat: event.repeat,
                    keycode: event.physical_key.to_scancode().unwrap()
                })),
                ElementState::Released => Some(Event::KeyUp(KeyUpEvent {
                    key,
                    modifiers,
                    keycode: event.physical_key.to_scancode().unwrap()
                })),
            }
        },
        
        WindowEvent::Ime(winit::event::Ime::Commit(s)) => {
            Some(Event::TextInput(s.clone()))
        }

        WindowEvent::Focused(true) => Some(Event::Focus(FocusEvent::Gained)),
        WindowEvent::Focused(false) => Some(Event::Focus(FocusEvent::Lost)),

        // Lifecycle events are handled directly in app.rs, not here.
        // Gamepad events come from DeviceEvent, not WindowEvent — handled separately.
        _ => None,
    }
}

/// Translate a winit `PhysicalKey` to a Phosphor `Key`.
pub fn translate_key(key: &PhysicalKey) -> Key {
    match key {
        PhysicalKey::Code(code) => match code {
            KeyCode::ArrowUp    => Key::ArrowUp,
            KeyCode::ArrowDown  => Key::ArrowDown,
            KeyCode::ArrowLeft  => Key::ArrowLeft,
            KeyCode::ArrowRight => Key::ArrowRight,
            KeyCode::Enter      => Key::Enter,
            KeyCode::Backspace  => Key::Backspace,
            KeyCode::Delete     => Key::Delete,
            KeyCode::Tab        => Key::Tab,
            KeyCode::Escape     => Key::Escape,
            KeyCode::Space      => Key::Space,
            KeyCode::ShiftLeft | KeyCode::ShiftRight   => Key::Shift,
            KeyCode::ControlLeft | KeyCode::ControlRight => Key::Ctrl,
            KeyCode::AltLeft | KeyCode::AltRight       => Key::Alt,
            KeyCode::SuperLeft | KeyCode::SuperRight   => Key::Meta,
            KeyCode::F1  => Key::F(1),
            KeyCode::F2  => Key::F(2),
            KeyCode::F3  => Key::F(3),
            KeyCode::F4  => Key::F(4),
            KeyCode::F5  => Key::F(5),
            KeyCode::F6  => Key::F(6),
            KeyCode::F7  => Key::F(7),
            KeyCode::F8  => Key::F(8),
            KeyCode::F9  => Key::F(9),
            KeyCode::F10 => Key::F(10),
            KeyCode::F11 => Key::F(11),
            KeyCode::F12 => Key::F(12),
            KeyCode::KeyA => Key::Char('a'),
            KeyCode::KeyB => Key::Char('b'),
            KeyCode::KeyC => Key::Char('c'),
            KeyCode::KeyD => Key::Char('d'),
            KeyCode::KeyE => Key::Char('e'),
            KeyCode::KeyF => Key::Char('f'),
            KeyCode::KeyG => Key::Char('g'),
            KeyCode::KeyH => Key::Char('h'),
            KeyCode::KeyI => Key::Char('i'),
            KeyCode::KeyJ => Key::Char('j'),
            KeyCode::KeyK => Key::Char('k'),
            KeyCode::KeyL => Key::Char('l'),
            KeyCode::KeyM => Key::Char('m'),
            KeyCode::KeyN => Key::Char('n'),
            KeyCode::KeyO => Key::Char('o'),
            KeyCode::KeyP => Key::Char('p'),
            KeyCode::KeyQ => Key::Char('q'),
            KeyCode::KeyR => Key::Char('r'),
            KeyCode::KeyS => Key::Char('s'),
            KeyCode::KeyT => Key::Char('t'),
            KeyCode::KeyU => Key::Char('u'),
            KeyCode::KeyV => Key::Char('v'),
            KeyCode::KeyW => Key::Char('w'),
            KeyCode::KeyX => Key::Char('x'),
            KeyCode::KeyY => Key::Char('y'),
            KeyCode::KeyZ => Key::Char('z'),
            KeyCode::Digit0 => Key::Char('0'),
            KeyCode::Digit1 => Key::Char('1'),
            KeyCode::Digit2 => Key::Char('2'),
            KeyCode::Digit3 => Key::Char('3'),
            KeyCode::Digit4 => Key::Char('4'),
            KeyCode::Digit5 => Key::Char('5'),
            KeyCode::Digit6 => Key::Char('6'),
            KeyCode::Digit7 => Key::Char('7'),
            KeyCode::Digit8 => Key::Char('8'),
            KeyCode::Digit9 => Key::Char('9'),
            other => Key::Other(format!("{other:?}")),
        },
        PhysicalKey::Unidentified(n) => Key::Other(format!("{n:?}")),
    }
}