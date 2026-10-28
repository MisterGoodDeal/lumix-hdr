#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod error;
mod history;
mod plan;
mod runner;
mod session;
mod state;

pub use error::CaptureError;
pub use history::{
    delete_session, list_session_summaries, load_session_detail, SessionDetail, SessionSummary,
};
pub use plan::{
    build_plan, BracketPlan, BracketRequest, ExposureSlot, ALLOWED_FRAME_COUNTS, MAX_CENTER_EV,
    MAX_STEP_EV, MIN_CENTER_EV, MIN_STEP_EV,
};
pub use runner::{preview_plan, run_capture, CaptureProgress};
pub use session::{relocate_session, CaptureSessionResult, FileEntry, SessionManifest};
pub use state::{CaptureState, CaptureStep};
