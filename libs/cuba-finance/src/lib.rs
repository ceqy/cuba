//! Cuba Finance - Shared library for FI module services
//!
//! This library provides common functionality used across Financial (FI) services:
//! - AP Service (Accounts Payable)
//! - AR Service (Accounts Receivable)
//! - CO Service (Cost Control)
//! - TR Service (Treasury Management)
//!
//! # Features
//! - GL Client: Unified gRPC client for General Ledger integration
//! - Common data structures and utilities

pub mod gl_client;

// Re-export commonly used items
pub use gl_client::{GlClient, GlLineItem, create_gl_client};
