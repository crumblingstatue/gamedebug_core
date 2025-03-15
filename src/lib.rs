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
pub static IMMEDIATE: MsgBuf<String> = MsgBuf::new(false);
/// Persistent messages that last between frames
pub static PERSISTENT: MsgBuf<PerEntry> = MsgBuf::new(false);

/// A statically globally accessible message buffer
pub struct MsgBuf<Msg> {
    msgs: Mutex<Vec<Msg>>,
    enabled: AtomicBool,
}

impl<Msg> MsgBuf<Msg> {
    /// Create a new empty message buffer
    pub const fn new(enabled: bool) -> Self {
        Self {
            msgs: Mutex::new(Vec::new()),
            enabled: AtomicBool::new(enabled),
        }
    }
    /// Push a message to the buffer
    pub fn push(&self, msg: Msg) {
        if self.enabled.load(Ordering::Acquire) {
            self.msgs.lock().unwrap().push(msg);
        }
    }
    /// Toggle whether the buffer is "enabled". If it's not enabled, pushing won't do anything.
    pub fn toggle(&self) {
        let current = self.enabled.load(Ordering::Acquire);
        self.enabled.store(!current, Ordering::Release);
    }
    /// Clear the buffer completely
    pub fn clear(&self) {
        self.msgs.lock().unwrap().clear();
    }
    /// Returns whether the buffer is enabled
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
    /// Sets the enabled status of the buffer
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release)
    }
    /// Removes old messages from the buffer, until it's `max` length.
    pub fn trim_old(&self, max: usize) {
        let mut msgs = self.msgs.lock().unwrap();
        while msgs.len() > max {
            msgs.remove(0);
        }
    }
    /// Executes a function for each message in the buffer.
    pub fn for_each(&self, mut f: impl FnMut(&Msg)) {
        for en in &*(self.msgs.lock().unwrap()) {
            f(en)
        }
    }
    /// Returns the length of the message buffer
    pub fn len(&self) -> usize {
        self.msgs.lock().unwrap().len()
    }
    /// Returns whether the message buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Persistent info entry with a frame stamp
pub struct PerEntry {
    /// The frame this information was recorded on
    pub frame: u32,
    /// The [`Info`]
    pub info: String,
    /// Source code location of the entry, if any
    pub src_loc: Option<SrcLoc>,
}

/// Source code location of an entry
#[allow(missing_docs)]
pub struct SrcLoc {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[macro_export]
#[doc(hidden)]
macro_rules! _gamedebug_core_src_loc {
    () => {
        $crate::SrcLoc {
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}

/// Add persistent information
pub fn per(info: String, src_loc: Option<SrcLoc>) {
    PERSISTENT.push(PerEntry {
        frame: frame(),
        info,
        src_loc,
    });
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
        $crate::IMMEDIATE.push(format!($($arg)*));
    }};
}

/// `dbg!`-like macro for pushing an immediate message
#[macro_export]
macro_rules! imm_dbg {
    ($val:expr $(,)?) => {{
        if $crate::IMMEDIATE.enabled() {
            $crate::IMMEDIATE.push(format!(
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
        $crate::per(format!($($arg)*), Some($crate::_gamedebug_core_src_loc!()));
    }};
}

/// `dbg!`-like macro for pushing a persistent message
#[macro_export]
macro_rules! per_dbg {
    ($x:expr) => {{
        $crate::per(
            format!(concat!(stringify!($x), ": {:#?}"), $x),
            Some($crate::_gamedebug_core_src_loc!()),
        );
        $x
    }};
}

#[test]
fn basic_macro_sanity_test() {
    per!("Hi!");
    per_dbg!(42);
}
