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
#[cfg_attr(test, derive(PartialEq, Eq))]
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
#[cfg_attr(test, derive(Debug, PartialEq))]
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

/// `println!`-like macro for pushing an immediate message
#[macro_export]
macro_rules! imm {
    ($($arg:tt)*) => {{
        $crate::imm($crate::Info::Msg(format!($($arg)*)));
    }};
}

/// `dbg!`-like macro for pushing an immediate message
#[macro_export]
macro_rules! imm_dbg {
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

/// `println!`-like macro for pushing a persistent message
#[macro_export]
macro_rules! per {
    ($($arg:tt)*) => {{
        $crate::per($crate::Info::Msg(format!($($arg)*)));
    }};
}

/// `dbg!`-like macro for pushing a persistent message
#[macro_export]
macro_rules! per_dbg {
    ($x:expr) => {{
        $crate::per($crate::Info::Msg(format!(
            concat!(stringify!($x), ": {:#?}"),
            $x
        )));
        $x
    }};
}

#[test]
fn test_per() {
    per!("Hello");
    let _n: i32 = per_dbg!(2 * 8) * 4;
    assert_eq!(
        PERSISTENT.lock().unwrap()[0].info,
        Info::Msg("Hello".into())
    );
    assert_eq!(
        PERSISTENT.lock().unwrap()[1].info,
        Info::Msg("2 * 8: 16".into())
    );
    toggle();
    imm!("Hi");
    let _n: i32 = imm_dbg!(2 * 8) * 4;
    assert_eq!(IMMEDIATE.lock().unwrap()[0], Info::Msg("Hi".into()));
    assert_eq!(IMMEDIATE.lock().unwrap()[1], Info::Msg("2 * 8: 16".into()));
}
