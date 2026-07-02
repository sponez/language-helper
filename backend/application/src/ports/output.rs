//! Outbound ports used by application use cases.

pub mod repository;

pub use repository::{
    CardRepository, LanguageProfileRepository, StudySessionRepository, UserRepository,
};
