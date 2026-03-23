//! `mori-context` -- Build LLM context from the code index.
//!
//! Takes search queries against `mori-index`, assembles matching symbols and
//! their source code into structured context blocks for LLM consumption.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod assemble;
mod error;
mod query;
mod snippet;

pub use assemble::{ContextBlock, ContextResponse, assemble};
pub use error::ContextError;
pub use query::{ContextQuery, SearchStrategy};
pub use snippet::SnippetConfig;
