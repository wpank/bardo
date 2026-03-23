//! Bardo Golem Mortality Engine
//!
//! Three independent death clocks (economic, epistemic, stochastic),
//! composite vitality state, behavioral phases, knowledge demurrage,
//! fractal mortality levels, and mortal memory integration with the Grimoire.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod clock;
pub mod demurrage;
pub mod economic;
pub mod epistemic;
pub mod error;
pub mod fractal;
pub mod mortal_memory;
pub mod stochastic;
pub mod vitality;

pub use clock::{ClockContext, ClockEvent, DeathCause, MortalityClock};
pub use economic::EconomicClock;
pub use epistemic::EpistemicClock;
pub use error::MortalityError;
pub use stochastic::StochasticClock;
pub use vitality::{VitalityConfig, VitalityState};
