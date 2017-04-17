//! # sixaxis-rs
//! Rust library for using the Sony DUALSHOCK3/SIXAXIS controller under Linux.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

extern crate byteorder;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path;
use std::sync::{Arc, Mutex};
use std::thread;

use byteorder::{ByteOrder, NativeEndian};

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Represents the various analog controls available.
/// Yes, for some reason the SIXAXIS has four axes.
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

/// Represents the current state of the SIXAXIS controller, including
/// the position of all analog axes and the state of all digital buttons.
pub struct State {
    axes: HashMap<Axis, i16>,
    shoulders: HashMap<Shoulder, u16>,
    buttons: HashMap<Button, bool>,
}

/// Represents a DUALSHOCK3/SIXAXIS controller connected
/// as a Linux input device (e.g. /dev/input/js0)
pub struct SixAxis {
    /// Path we opened (for debug)
    path: path::PathBuf,
    /// The current state, shared with the read thread
    state: Arc<Mutex<State>>,
    /// The read thread, which blocks on the event
    child: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoController,
    UnknownError,
    NotImplemented,
    NotOpen,
    AlreadyOpen,
    IOError,
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

enum Event {
    Axis(Axis, i16),
    Shoulder(Shoulder, u16),
    Button(Button, bool)
}

// ****************************************************************************
//
// Private Data
//
// ****************************************************************************

const EVENT_SIZE:usize = 8;
const VERBOSE:bool = false;

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
            state: Arc::new(Mutex::new(State::new())),
            child: None
        }
    }

    /// Actually open the path to the controller.
    pub fn open(&mut self) -> Result<()> {
        // Open the file.
        // This is moved to the thread.
        let mut f = fs::File::open(&self.path)?;
        // Clone the Arc holding the state.
        // This is moved to the thread.
        let state_ref = self.state.clone();
        // Make the thread to read the file in a blocking fashion
        self.child = Some(thread::spawn(move || {
            loop {
                let mut buf = [0u8; EVENT_SIZE];
                match f.read_exact(&mut buf) {
                    Ok(_) => {
                        let ev = process_event(&buf);
                        let mut state = state_ref.lock().unwrap();
                        match ev {
                            Ok(Event::Axis(axis, value)) => { state.axes.insert(axis, value); }
                            Ok(Event::Shoulder(shoulder, value)) => { state.shoulders.insert(shoulder, value); }
                            Ok(Event::Button(button, value)) => { state.buttons.insert(button, value); }
                            // Drop event
                            Err(_) => {},
                        }
                    }
                    Err(_) => break,
                };
            }
            println!("Bluetooth read thread exited!");
        }));
        Ok(())
    }

    /// Close the controller.
    ///
    /// Can call `open` later, if required.
    pub fn close(&mut self) -> Result<()> {
        match self.child {
            None => Err(Error::NotOpen),
            Some(ref th) => {
                // Kill the thread
                Err(Error::NotImplemented)
            }
        }
    }

    /// Read a thumb-stick axis.
    ///
    /// Returns the most recent value from the controller.
    /// The thumb sticks are -32768..+32767. Returns 0
    /// if the axis has never reported itself.
    pub fn read_axis(&self, axis: Axis) -> Result<i16> {
        // Return error if thread is dead
        let state = self.state.lock().unwrap();
        match state.axes.get(&axis) {
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
        match state.shoulders.get(&shoulder) {
            Some(value) => Ok(*value),
            None => Ok(0)
        }
    }

    /// Read a digital button.
    ///
    /// Returns the most recent value from the controller.
    /// `true` means pressed and `false` means not pressed.
    /// Returns `false` if the button has never reported itself.
    pub fn read_button(&self, button: Button) -> Result<bool> {
        // Return error if thread is dead
        let state = self.state.lock().unwrap();
        match state.buttons.get(&button) {
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

const EVENT_TYPE_BUTTON:u8 = 1;
const EVENT_TYPE_STICK:u8 = 2;
const EVENT_TYPE_INIT:u8 = 128;
const EVENT_TYPE_INITBUTTON:u8 = 129;
const EVENT_TYPE_INITSTICK:u8 = 130;

const EVENT_STICK_IDX_LX:u8 = 0;
const EVENT_STICK_IDX_LY:u8 = 1;
const EVENT_STICK_IDX_RX:u8 = 2;
const EVENT_STICK_IDX_RY:u8 = 3;
const EVENT_STICK_IDX_L2:u8 = 12;
const EVENT_STICK_IDX_R2:u8 = 13;
const EVENT_STICK_IDX_L1:u8 = 14;
const EVENT_STICK_IDX_R1:u8 = 15;

const EVENT_BUTTON_IDX_SELECT:u8 = 0;
const EVENT_BUTTON_IDX_LEFTSTICK:u8 = 1;
const EVENT_BUTTON_IDX_RIGHTSTICK:u8 = 2;
const EVENT_BUTTON_IDX_START:u8 = 3;
const EVENT_BUTTON_IDX_UP:u8 = 4;
const EVENT_BUTTON_IDX_RIGHT:u8 = 5;
const EVENT_BUTTON_IDX_DOWN:u8 = 6;
const EVENT_BUTTON_IDX_LEFT:u8 = 7;
const EVENT_BUTTON_IDX_L2:u8 = 8;
const EVENT_BUTTON_IDX_R2:u8 = 9;
const EVENT_BUTTON_IDX_L1:u8 = 10;
const EVENT_BUTTON_IDX_R1:u8 = 11;
const EVENT_BUTTON_IDX_PS:u8 = 16;
const EVENT_BUTTON_IDX_TRIANGLE:u8 = 12;
const EVENT_BUTTON_IDX_CIRCLE:u8 = 13;
const EVENT_BUTTON_IDX_CROSS:u8 = 14;
const EVENT_BUTTON_IDX_SQUARE:u8 = 15;

fn process_event(buf: &[u8; EVENT_SIZE]) -> Result<Event> {
    let timestamp = NativeEndian::read_u32(&buf[0..3]);
    let value = NativeEndian::read_u16(&buf[4..5]);
    let ev_type = buf[6];
    let ev_idx = buf[7];
    match ev_type {
        EVENT_TYPE_STICK | EVENT_TYPE_INITSTICK => process_stick(ev_idx, value),
        EVENT_TYPE_BUTTON | EVENT_TYPE_INITBUTTON => process_button(ev_idx, value),
        _ => Err(Error::UnknownError),
    }
}

fn process_stick(ev_idx: u8, value: u16) -> Result<Event> {
    let s_val:i16 = if (value & 0x8000) != 0 {
        ((value as i32) - 65536) as i16
    } else {
        value as i16
    };
    Ok(Event::Axis(Axis::LX, 0))
}

fn process_button(ev_idx: u8, value: u16) -> Result<Event> {
    Ok(Event::Button(Button::Start, false))
}

impl State {
    fn new() -> State {
        State {
            axes: HashMap::new(),
            shoulders: HashMap::new(),
            buttons: HashMap::new(),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(_e: ::std::io::Error) -> Error {
        Error::IOError
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
