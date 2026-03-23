//! Bankr provider domain modules -- metabolic loop, credit management,
//! model routing, cross-model verification.

pub mod config;
pub mod credits;
pub mod metabolic;
pub mod routing;
pub mod verification;

pub use config::{AutoReplenishPolicy, BankrProviderConfig, SelfFundingConfig, ThrottlePolicy};
pub use credits::{ConversionRate, CreditBalance, VaultFeeConverter};
pub use metabolic::{MetabolicLoopConfig, MetabolicLoopMonitor, SustainabilityRatio};
pub use routing::{BankrModelMapping, BankrModelTier, select_bankr_model};
pub use verification::{CrossModelVerification, ParsedAction, VerificationResponse};
