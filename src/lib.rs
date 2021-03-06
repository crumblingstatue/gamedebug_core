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
pub static IMMEDIATE: Mutex<Vec<Info>> = Mutex::new(Vec::new());
/// Persistent messages that last between frames
pub static PERSISTENT: Mutex<Vec<PerEntry>> = Mutex::new(Vec::new());

static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Generic RGBA color struct for coloring certain debug information
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
    /// Alpha component
    pub a: u8,
}

/// Debug information
pub enum Info {
    /// A text message
    Msg(String),
    /// A rectangle to be drawn on the screen
    Rect(f32, f32, f32, f32, Color),
}

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Add immediate info for the current frame
pub fn imm(info: Info) {
    if ENABLED.load(Ordering::Acquire) {
        IMMEDIATE.lock().unwrap().push(info);
    }
}

/// Persistent info entry with a frame stamp
pub struct PerEntry {
    /// The frame this information was recorded on
    pub frame: u32,
    /// The [`Info`]
    pub info: Info,
}

/// Add persistent information
pub fn per(info: Info) {
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

/// Increment the frame counter. Do this every frame.
pub fn inc_frame() {
    let frame = FRAME_COUNTER.load(Ordering::Acquire);
    FRAME_COUNTER.store(frame + 1, Ordering::Release);
}

/// Query the frame counter
pub fn frame() -> u32 {
    FRAME_COUNTER.load(Ordering::Acquire)
}

/// Show an expression as an immediate message
#[macro_export]
macro_rules! imm_msg {
    ($x:expr) => {{
        if $crate::enabled() {
            $crate::imm($crate::Info::Msg(format!(
                concat!(stringify!($x), ": {:#?}"),
                $x
            )));
        }
        $x
    }};
}

/// Format and record a persistent message. It works like `println!`.
#[macro_export]
macro_rules! per_msg {
    ($($arg:tt)*) => {{
        $crate::per($crate::Info::Msg(format!($($arg)*)));
    }};
}
