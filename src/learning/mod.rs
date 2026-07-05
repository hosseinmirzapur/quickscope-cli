//! Learning Engine — auto-tuner + LLM post-mortem.
//!
//! Two-pronged:
//! 1. Auto-Tuner: statistical discrimination analysis on closed trades,
//!    nudges weights ±5% with guard rails.
//! 2. LLM Post-Mortem: on-demand review by external LLM provider
//!    (OpenAI, Anthropic, or local Ollama).

pub mod analyzer;
pub mod tuner;
pub mod journal;
pub mod llm;

pub use analyzer::{analyze_discrimination, FeatureDiscrimination};
pub use tuner::run_auto_tune;
pub use journal::run_post_mortem;
pub use llm::LlmProvider;