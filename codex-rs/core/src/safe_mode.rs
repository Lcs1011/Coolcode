use std::sync::OnceLock;

static SAFE_MODE: OnceLock<bool> = OnceLock::new();

pub fn init(enabled: bool) {
    SAFE_MODE
        .set(enabled)
        .expect("SafeMode was already initialized");
}

pub fn enabled() -> bool {
    SAFE_MODE.get().copied().unwrap_or(true)
}
