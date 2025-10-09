//! API layer for Language Helper application.
//!
//! This library defines the API interfaces and data transfer objects (DTOs) used
//! to communicate between the GUI layer and the core business logic layer.
//!
//! # Architecture
//!
//! The API layer serves as a contract between the presentation layer (GUI) and
//! the core business logic. It defines:
//!
//! - **apis**: Trait definitions for various API domains (e.g., UsersApi)
//! - **app_api**: Main application API that aggregates all domain APIs
//! - **models**: Data transfer objects (DTOs) for API communication
//! - **errors**: API-specific error types
//!
//! # Design Philosophy
//!
//! This layer:
//! - Defines interfaces as traits for flexibility and testability
//! - Uses DTOs to decouple API contracts from domain models
//! - Provides clear error types for API consumers
//! - Remains agnostic to the underlying implementation
//!
//! # Example Usage
//!
//! ```no_run
//! use lh_api::app_api::AppApi;
//! use lh_api::apis::user_api::UsersApi;
//!
//! fn list_users(app_api: &dyn AppApi) {
//!     match app_api.users_api().get_usernames() {
//!         Ok(usernames) => {
//!             for username in usernames {
//!                 println!("User: {}", username);
//!             }
//!         }
//!         Err(e) => eprintln!("Error: {:?}", e),
//!     }
//! }
//! ```

pub mod apis;
pub mod app_api;
pub mod errors;
pub mod models;
