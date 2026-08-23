//! Course identity, requirement matching, pools/concentrations, and also-offered/mutex behavior.

#[path = "../common/helpers.rs"]
mod helpers;
pub use helpers::*;

mod course_identity;
mod course_relations;
mod pools;
mod requirements;
