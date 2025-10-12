//! Core business logic layer.
//!
//! This crate contains the core domain models, business logic, and repository
//! trait definitions. It is independent of any specific persistence implementation.
//!
//! # Architecture
//!
//! The core layer follows clean architecture principles:
//!
//! - **Models**: Contains the core business entities (e.g., User)
//! - **Repositories**: Defines traits for persistence operations
//! - **Services**: Implements business logic using repository traits
//! - **Errors**: Defines core error types
//!
//! # Design Philosophy
//!
//! The core layer:
//! - Contains pure business logic
//! - Is independent of frameworks and external libraries
//! - Defines interfaces (traits) for persistence without implementing them
//! - Uses dependency injection to receive repository implementations
//!
//! # Example Usage
//!
//! ```no_run
//! use lh_core::services::user_service::UserService;
//! use lh_core::repositories::user_repository::UserRepository;
//!
//! // Your persistence layer implements UserRepository
//! async fn example(repository: impl UserRepository) {
//!     let user_service = UserService::new(repository);
//!
//!     match user_service.get_all_usernames().await {
//!         Ok(usernames) => {
//!             for username in usernames {
//!                 println!("User: {}", username);
//!             }
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

pub mod api_impl;
pub mod models;
pub mod errors;
pub mod repositories;
pub mod services;
