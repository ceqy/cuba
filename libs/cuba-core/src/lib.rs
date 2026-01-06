use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

// Re-export commonly used types for downstream crates
pub use serde::{Deserialize, Serialize};

// --- Core Domain Primitives ---

pub trait Aggregate {
    type Id;
    fn id(&self) -> &Self::Id;
    fn version(&self) -> u64;
    fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>>;
}

pub trait DomainEvent: erased_serde::Serialize + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> Uuid;
}
erased_serde::serialize_trait_object!(DomainEvent);

// --- CQRS Primitives ---

#[async_trait]
pub trait Command: Send + Sync {}

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<()>;
}

#[async_trait]
pub trait Query: Send + Sync {
    type Result: Send;
}

#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Result>;
}

// --- Repository Primitives ---

#[async_trait]
pub trait Repository<A: Aggregate>: Send + Sync {
    async fn save(&self, aggregate: &mut A) -> Result<()>;
    async fn find_by_id(&self, id: &A::Id) -> Result<Option<A>>;
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, events: Vec<Box<dyn DomainEvent>>) -> Result<()>;
}
