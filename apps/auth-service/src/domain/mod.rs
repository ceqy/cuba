//! Auth Service - Domain Layer
//!
//! 领域层包含业务核心逻辑，不依赖任何外部框架。

pub mod aggregates;
pub mod errors;
pub mod events;
pub mod repositories;
pub mod services;
pub mod value_objects;

pub use errors::DomainError;
