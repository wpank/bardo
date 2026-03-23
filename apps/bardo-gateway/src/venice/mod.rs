//! Venice integration -- security classification, DIEM budget tracking,
//! and privacy-aware routing for zero-retention inference.

pub mod config;
pub mod diem;
pub mod error;
pub mod router;
pub mod security_class;

pub use config::{VeniceModelMapping, VeniceParameters, VeniceProviderConfig, WebSearchMode};
pub use diem::{DiemAllocations, DiemCategory, DiemConfig, DiemTracker};
pub use error::VeniceError;
pub use router::{RouterConfig, SecurityClassRouter};
pub use security_class::{ClassificationResult, SecurityClass, SecurityTrigger};
