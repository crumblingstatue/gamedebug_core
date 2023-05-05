//! Reusable game debug library, providing both immediate and persistent message buffers
//!
//! The information is stored in global variables, so the debug information can be provided
//! anywhere in the code.
//!
//! This is just a core library, to actually show anything on the screen, you have to write
//! code that reads the information contained in this library and presents it.
#![warn(missing_docs)]

use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Mutex,
};

/// Immediate messages that are for this frame only
pub static IMMEDIATE: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// Persistent messages that last between frames
pub static PERSISTENT: Mutex<Vec<PerEntry>> = Mutex::new(Vec::new());

static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Add immediate info for the current frame
pub fn imm(info: String) {
    if ENABLED.load(Ordering::Acquire) {
        IMMEDIATE.lock().unwrap().push(info);
    }
}

/// Persistent info entry with a frame stamp
pub struct PerEntry {
    /// The frame this information was recorded on
    pub frame: u32,
    /// The [`Info`]
    pub info: String,
}

/// Add persistent information
pub fn per(info: String) {
    let mut log = PERSISTENT.lock().unwrap();
    log.push(PerEntry {
        frame: FRAME_COUNTER.load(Ordering::Acquire),
        info,
    });
    if log.len() > 20 {
        log.remove(0);
    }
}

/// Clear the immediate debug information. Do this every frame after presenting the info.
pub fn clear_immediates() {
    IMMEDIATE.lock().unwrap().clear();
}

/// Toggle the debug overlay
pub fn toggle() {
    let current = ENABLED.load(Ordering::Acquire);
    ENABLED.store(!current, Ordering::Release);
}

/// Whether the debug overlay is enabled
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

/// Set whether the debug overlay is enabled or not
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Release)
}

/// Increment the frame counter. Do this every frame.
pub fn inc_frame() {
    let frame = FRAME_COUNTER.load(Ordering::Acquire);
    FRAME_COUNTER.store(frame + 1, Ordering::Release);
}

/// Query the frame counter
pub fn frame() -> u32 {
    FRAME_COUNTER.load(Ordering::Acquire)
}

/// Execute a function for each immediate item
pub fn for_each_imm(mut f: impl FnMut(&String)) {
    for info in &*(IMMEDIATE.lock().unwrap()) {
        f(info)
    }
}

/// Execute a function for each persistent item
pub fn for_each_per(mut f: impl FnMut(&PerEntry)) {
    for en in &*(PERSISTENT.lock().unwrap()) {
        f(en)
    }
}

/// `println!`-like macro for pushing an immediate message
#[macro_export]
macro_rules! imm {
    ($($arg:tt)*) => {{
        $crate::imm(format!($($arg)*));
    }};
}

/// `dbg!`-like macro for pushing an immediate message
#[macro_export]
macro_rules! imm_dbg {
    ($val:expr $(,)?) => {{
        if $crate::enabled() {
            $crate::imm(format!(
                concat!(file!(), ":", line!(), ": ", stringify!($val), ": {:#?}"),
                $val
            ));
        }
        $val
    }};
    ($($val:expr),+ $(,)?) => {{
        $($crate::imm_dbg!($val);)+
    }}
}

/// `println!`-like macro for pushing a persistent message
#[macro_export]
macro_rules! per {
    ($($arg:tt)*) => {{
        $crate::per(format!($($arg)*));
    }};
}

/// `dbg!`-like macro for pushing a persistent message
#[macro_export]
macro_rules! per_dbg {
    ($x:expr) => {{
        $crate::per(format!(concat!(stringify!($x), ": {:#?}"), $x));
        $x
    }};
}
