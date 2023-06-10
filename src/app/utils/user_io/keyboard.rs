use { crate::prelude::*, super::KeyState };



lazy_static! {
    pub static ref RELEASED_KEYS: RwLock<HashSet<Key>> = RwLock::new(HashSet::new());
}

pub const N_KEYS: usize = 163;
pub static INPUTS: [Atomic<KeyState>; N_KEYS] = const_default();

macros::atomic_static! {
    pub static IS_CAPTURED: bool = false;
}

pub fn set_input_capture(capture: bool) {
    IS_CAPTURED.store(capture, Relaxed);
}

pub fn press(key: Key) {
    INPUTS[key as usize].store(KeyState::Pressed, Release);
}

pub fn release(key: Key) {
    INPUTS[key as usize].store(KeyState::Released, Release);
}

pub fn is_pressed(key: Key) -> bool {
    ensure_or!(!IS_CAPTURED.load(Relaxed), return false);
    INPUTS[key as usize].load(Acquire).is_pressed()
}

pub fn just_pressed(key: Key) -> bool {
    let pressed = is_pressed(key);
    if pressed { RELEASED_KEYS.write().insert(key); }
    pressed
}

pub fn is_pressed_combo<'k>(keys: impl IntoIterator<Item = &'k Key>) -> bool {
    ensure_or!(!IS_CAPTURED.load(Relaxed), return false);
    keys.into_iter().all(|&key| INPUTS[key as usize].load(Acquire).is_pressed())
}

pub fn just_pressed_combo<'k>(keys: impl IntoIterator<Item = &'k Key>) -> bool {
    ensure_or!(!IS_CAPTURED.load(Relaxed), return false);

    let mut released_keys = RELEASED_KEYS.write();

    keys.into_iter().all(|&key| {
        let pressed = INPUTS[key as usize].load(Acquire).is_pressed();
        if pressed { released_keys.insert(key); }
        pressed
    })
}

pub fn update() {
    for key in RELEASED_KEYS.write().drain() { release(key); }
}