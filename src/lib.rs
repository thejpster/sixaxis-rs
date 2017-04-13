//! # sixaxis-rs
//! Rust library for using the Sony DUALSHOCK3/SIXAXIS controller under Linux.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use std::path;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Represents the various analog controls available.
/// Yes, for some reason the SixAxis has four axes.
pub enum Axis {
    /// Left thumb stick, left/right
    LX,
    /// Left thumb stick, up/down
    LY,
    /// Right thumb stick, left/right
    RX,
    /// Right thumb stick, up/down
    RY,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Represents the shoulder buttons, which are analog
/// but go from 0 .. 65535.
pub enum Shoulder {
    /// Upper left shoulder
    L1,
    /// Lower left shoulder
    L2,
    /// Upper right shoulder
    R1,
    /// Lower right shoulder
    R2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Represents the digital buttons available
pub enum Button {
    Square,
    Circle,
    Triangle,
    Cross,
    PS,
    Start,
    Select,
    LeftStick,
    RightStick,
    Up,
    Down,
    Left,
    Right,
    L1,
    L2,
    R1,
    R2,
}

/// Represents the current state of the SixAxis controller, including
/// the position of all analog axes and the state of all digital buttons.
pub struct State {
    analog: HashMap<Axis, i16>,
    shoulder: HashMap<Shoulder, u16>,
    digital: HashMap<Button, bool>,
}

/// Represents a DUALSHOCK3/SIXAXIS controller connected
/// as a Linux input device (e.g. /dev/input/js0)
pub struct SixAxis {
    /// Path we opened (for debug)
    path: path::PathBuf,
    /// The current state, shared with the read thread
    state: Arc<Mutex<State>>,
    // /// The read thread, which blocks on the event
    // child: Option<thread::Thread>,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoController,
    UnknownError,
    NotImplemented,
    NotOpen,
    AlreadyOpen
}

pub type Result<T> = ::std::result::Result<T, Error>;

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Private Types
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Private Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

impl SixAxis {

    /// Create a new SixAxis object, but don't open the file
    /// just yet.
    pub fn new<P: AsRef<path::Path>>(path: P) -> SixAxis {
        // Init the state
        SixAxis {
            path: path::PathBuf::from(path.as_ref()),
            state: Arc::new(Mutex::new(State::new()))
        }
    }

    /// Actually open the path to the controller.
    pub fn open(&mut self) -> Result<()> {
        // Open the file
        // todo
        // Make the thread to read the file
        // todo
        Err(Error::NotImplemented)
    }

    /// Close the controller.
    ///
    /// Can call `open` later, if required.
    pub fn close(&mut self) -> Result<()> {
        Err(Error::NotImplemented)
    }

    /// Read an analog axis.
    ///
    /// Returns the most recent value from the controller.
    /// The thumb sticks are -32768..+32767. Returns 0
    /// if the axis has never reported itself.
    pub fn read_analog(&self, axis: Axis) -> Result<i16> {
        // Return error if thread is dead
        let state = self.state.lock().unwrap();
        match state.analog.get(&axis) {
            Some(value) => Ok(*value),
            None => Ok(0)
        }
    }

    /// Read an analog shoulder button.
    ///
    /// Returns the most recent value from the controller.
    /// The shoulder buttons are 0..65535. Returns 0
    /// if the shoulder has never reported itself.
    pub fn read_shoulder(&self, shoulder: Shoulder) -> Result<u16> {
        // Return error if thread is dead
        let state = self.state.lock().unwrap();
        match state.shoulder.get(&shoulder) {
            Some(value) => Ok(*value),
            None => Ok(0)
        }
    }

    /// Read a digital button.
    ///
    /// Returns the most recent value from the controller.
    /// `true` means pressed and `false` means not pressed.
    /// Returns `false` if the button has never reported itself.
    pub fn read_digital(&self, button: Button) -> Result<bool> {
        // Return error if thread is dead
        let state = self.state.lock().unwrap();
        match state.digital.get(&button) {
            Some(value) => Ok(*value),
            None => Ok(false)
        }
    }
}

impl ::std::fmt::Debug for SixAxis {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "<SixAxis path={:?}>", self.path)
    }
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************


impl State {
    fn new() -> State {
        State {
            analog: HashMap::new(),
            shoulder: HashMap::new(),
            digital: HashMap::new(),
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
