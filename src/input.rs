use std::sync::RwLock;
use std::collections::HashMap;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use lazy_static::lazy_static;

pub struct Input(HashMap<Keycode, (bool, bool)>);

impl Input {
    pub fn new() -> Input {
        Input(HashMap::new())
    }

    pub fn key_pressed(&self, key: Keycode) -> bool {
        self.0.get(&key).map(|(p, _h)| *p).unwrap_or(false)
    }

    pub fn key_held(&self, key: Keycode) -> bool {
        self.0.get(&key).map(|(_p, h)| *h).unwrap_or(false)
    }

    pub fn update(&mut self, events: &[Event]) {
        for (_, (pressed, _held)) in self.0.iter_mut() {
            *pressed = false;
        }

        for event in events.iter() {
            match event {
                Event::KeyUp{ keycode: Some(k), .. } => {
                    *self.0.entry(*k).or_insert((false, false)) = (false, false);
                }
                Event::KeyDown{ keycode: Some(k), .. } => {
                    let entry = self.0.entry(*k).or_insert((false, false));
                    let pressed = !entry.1;
                    *entry = (pressed, true);
                }
                _ => {}
            }
        }
    }
}

pub fn update(events: &[Event]) {
    let mut input_state = INPUT_STATE.write().unwrap();
    input_state.update(events);
}

pub fn key_held(key: Keycode) -> bool {
    INPUT_STATE.read().unwrap().key_held(key)
}

pub fn key_pressed(key: Keycode) -> bool {
    INPUT_STATE.read().unwrap().key_pressed(key)
}

lazy_static! {
    static ref INPUT_STATE: RwLock<Input> = RwLock::new(Input::new());
}