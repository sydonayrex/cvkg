//! Frame phase types shared across all CVKG crates.

/// The ordered phases of a single CVKG render frame.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum FramePhase {
    Input,
    State,
    Layout,
    Animation,
    Render,
    Composite,
    PostFrame,
}

impl FramePhase {
    /// Advance to the next phase in the pipeline.
    pub fn next(self) -> Option<FramePhase> {
        match self {
            FramePhase::Input => Some(FramePhase::State),
            FramePhase::State => Some(FramePhase::Layout),
            FramePhase::Layout => Some(FramePhase::Animation),
            FramePhase::Animation => Some(FramePhase::Render),
            FramePhase::Render => Some(FramePhase::Composite),
            FramePhase::Composite => Some(FramePhase::PostFrame),
            FramePhase::PostFrame => None,
        }
    }

    /// Return a stable label for this phase.
    pub fn label(self) -> &'static str {
        match self {
            FramePhase::Input => "phase:Input",
            FramePhase::State => "phase:State",
            FramePhase::Layout => "phase:Layout",
            FramePhase::Animation => "phase:Animation",
            FramePhase::Render => "phase:Render",
            FramePhase::Composite => "phase:Composite",
            FramePhase::PostFrame => "phase:PostFrame",
        }
    }
}
