pub trait AssetManager: Send + Sync {
    /// Request an image asset. Returns the current state (Loading, Ready, or Error).
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>>;

    /// Pre-load an image into the cache.
    fn preload_image(&self, url: &str);
}

/// The phase of a touch or gesture event in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TouchPhase {
    /// The touch/gesture has just begun.
    Began,
    /// The touch/gesture is moving.
    Moved,
    /// The touch/gesture has ended normally.
    Ended,
    /// The touch/gesture was cancelled (e.g., by the system).
    Cancelled,
}

/// User input event types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    PointerDown {
        x: f32,
        y: f32,
        button: u32,
        proximity_field: f32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerUp {
        x: f32,
        y: f32,
        button: u32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerMove {
        x: f32,
        y: f32,
        proximity_field: f32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerClick {
        x: f32,
        y: f32,
        button: u32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerEnter,
    PointerLeave,
    /// Mouse wheel / trackpad scroll event.
    /// `delta_x` is the horizontal scroll amount, `delta_y` is the vertical scroll amount (positive = scroll down).
    PointerWheel {
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
        pointer_precision: f32,
    },
    /// Double-click event (rapid successive clicks).
    PointerDoubleClick {
        x: f32,
        y: f32,
        button: u32,
        pointer_precision: f32,
    },
    /// Drag-and-drop: drag started (pointer moved while button held past threshold).
    DragStart {
        x: f32,
        y: f32,
        button: u32,
        pointer_precision: f32,
    },
    /// Drag-and-drop: drag in progress.
    DragMove {
        x: f32,
        y: f32,
        pointer_precision: f32,
    },
    /// Drag-and-drop: drag ended (pointer released).
    DragEnd {
        x: f32,
        y: f32,
        pointer_precision: f32,
    },
    KeyDown {
        key: String,
        modifiers: KeyModifiers,
    },
    KeyUp {
        key: String,
        modifiers: KeyModifiers,
    },
    /// Focus gained by a node.
    FocusIn,
    /// Focus lost by a node.
    FocusOut,
    /// Clipboard copy event.
    Copy,
    /// Clipboard cut event.
    Cut,
    /// Clipboard paste event with the pasted text content.
    Paste(String),
    /// Input Method Editor event (e.g. CJK character composition)
    Ime(String),
    /// Touch began at the given position.
    TouchStart {
        x: f32,
        y: f32,
        touch_id: u64,
    },
    /// Touch moved to a new position.
    TouchMove {
        x: f32,
        y: f32,
        touch_id: u64,
    },
    /// Touch ended at the given position.
    TouchEnd {
        x: f32,
        y: f32,
        touch_id: u64,
    },
    /// Touch cancelled.
    TouchCancel {
        touch_id: u64,
    },
    /// Multi-touch pinch gesture.
    /// `center` is the gesture anchor point in device-independent pixels.
    /// `scale` is the relative pinch scale (>1 = expand, <1 = contract).
    /// `velocity` is the instantaneous velocity of the pinch.
    /// `phase` indicates the current phase of the gesture lifecycle.
    GesturePinch {
        center: [f32; 2],
        scale: f32,
        velocity: f32,
        phase: TouchPhase,
    },
    /// Multi-touch swipe/pan gesture.
    /// `direction` is the normalized direction vector [dx, dy].
    /// `velocity` is the instantaneous velocity of the swipe.
    /// `phase` indicates the current phase of the gesture lifecycle.
    GestureSwipe {
        direction: [f32; 2],
        velocity: f32,
        phase: TouchPhase,
    },
    /// Drag-and-drop: external file dropped onto window.
    FileDrop {
        x: f32,
        y: f32,
        path: String,
    },
    /// Gamepad connected.
    GamepadConnected {
        /// Gamepad device ID.
        id: u64,
        /// Human-readable name.
        name: String,
    },
    /// Gamepad disconnected.
    GamepadDisconnected {
        /// Gamepad device ID.
        id: u64,
    },
    /// Gamepad button pressed or released.
    GamepadButton {
        /// Gamepad device ID.
        id: u64,
        /// Button index.
        button: u32,
        /// Pressure in [0.0, 1.0].
        pressure: f32,
    },
    /// Gamepad axis moved.
    GamepadAxis {
        /// Gamepad device ID.
        id: u64,
        /// Axis index.
        axis: u32,
        /// Axis value in [-1.0, 1.0].
        value: f32,
    },
}

impl Event {
    /// Returns the input pointer precision value in physical pixels if applicable.
    ///
    /// WHY: Used to scale hit-testing bounding boxes for proximity matching.
    /// CONTRACT: Mouse pointer inputs return low precision values (close to 0.0px),
    /// whereas touch inputs return larger values (e.g., 150.0px) for finger emulation.
    pub fn pointer_precision(&self) -> f32 {
        match self {
            Self::PointerDown {
                pointer_precision, ..
            }
            | Self::PointerUp {
                pointer_precision, ..
            }
            | Self::PointerMove {
                pointer_precision, ..
            }
            | Self::PointerClick {
                pointer_precision, ..
            }
            | Self::PointerWheel {
                pointer_precision, ..
            }
            | Self::PointerDoubleClick {
                pointer_precision, ..
            }
            | Self::DragStart {
                pointer_precision, ..
            }
            | Self::DragMove {
                pointer_precision, ..
            }
            | Self::DragEnd {
                pointer_precision, ..
            } => *pointer_precision,
            _ => 0.0,
        }
    }

    /// Returns the canonical string name of the event for lookup in handler maps.
    pub fn name(&self) -> &'static str {
        match self {
            Self::PointerDown { .. } => "pointerdown",
            Self::PointerUp { .. } => "pointerup",
            Self::PointerMove { .. } => "pointermove",
            Self::PointerClick { .. } => "pointerclick",
            Self::PointerEnter => "pointerenter",
            Self::PointerLeave => "pointerleave",
            Self::PointerWheel { .. } => "pointerwheel",
            Self::PointerDoubleClick { .. } => "pointerdoubleclick",
            Self::DragStart { .. } => "dragstart",
            Self::DragMove { .. } => "dragmove",
            Self::DragEnd { .. } => "dragend",
            Self::KeyDown { .. } => "keydown",
            Self::KeyUp { .. } => "keyup",
            Self::FocusIn => "focusin",
            Self::FocusOut => "focusout",
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste(_) => "paste",
            Self::Ime(_) => "ime",
            Self::TouchStart { .. } => "touchstart",
            Self::TouchMove { .. } => "touchmove",
            Self::TouchEnd { .. } => "touchend",
            Self::TouchCancel { .. } => "touchcancel",
            Self::GesturePinch { .. } => "gesturepinch",
            Self::GestureSwipe { .. } => "gestureswipe",
            Self::FileDrop { .. } => "filedrop",
            Self::GamepadConnected { .. } => "gamepadconnected",
            Self::GamepadDisconnected { .. } => "gamepaddisconnected",
            Self::GamepadButton { .. } => "gamepadbutton",
            Self::GamepadAxis { .. } => "gamepadaxis",
        }
    }
}

/// Response from an event handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
    Handled,
    Ignored,
}

// =========================================================================
use crate::*;
use std::sync::Arc;

// P1-40: EventPhase -- documents event propagation phases
// =========================================================================
//
// The CVKG event system follows the standard capture/target/bubble
// model used by the W3C DOM Event spec. When an event fires, it
// propagates through 3 phases:
//
// 1. Capture: the event travels from the root down to the
//    target's parent. Listeners registered for the capture
//    phase fire first.
// 2. Target: the event reaches the target node itself. Listeners
//    on the target fire (regardless of capture/bubble).
// 3. Bubble: the event travels back up from the target's
//    parent to the root. Listeners registered for the bubble
//    phase fire last.
//
// Cancellation: any handler can call Event::stop_propagation()
// to prevent the event from continuing to the next phase or
// the next node. This affects only the current event instance.
//
// Example: a click on a button inside a panel:
//  - panel's capture handler fires
//  - button's capture handler fires
//  - button's target handler fires
//  - button's bubble handler fires
//  - panel's bubble handler fires
//
// Use this enum when registering listeners to specify which
// phase to listen for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPhase {
    /// Event is traveling from the root toward the target.
    Capture,
    /// Event has reached the target node.
    Target,
    /// Event is traveling from the target back toward the root.
    Bubble,
}

impl EventPhase {
    /// All phases in propagation order.
    pub const ALL: [EventPhase; 3] = [EventPhase::Capture, EventPhase::Target, EventPhase::Bubble];
}
