use async_trait::async_trait;
use serde::Serialize;
use std::fmt::Debug;

/// Command trait represents a write operation that changes state.
pub trait Command: Debug + Serialize + Send + Sync + 'static {}

/// Query trait represents a read operation that retrieves state without changing it.
pub trait Query: Debug + Serialize + Send + Sync + 'static {}

/// DomainEvent trait represents something that happened in the past.
pub trait DomainEvent: Debug + Serialize + Send + Sync + 'static {
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

/// Handler for commands.
#[async_trait]
pub trait CommandHandler<C: Command> {
    type Output;
    async fn handle(&self, command: C) -> anyhow::Result<Self::Output>;
}

/// Handler for queries.
#[async_trait]
pub trait QueryHandler<Q: Query> {
    type Output;
    async fn handle(&self, query: Q) -> anyhow::Result<Self::Output>;
}

/// Handler for domain events.
#[async_trait]
pub trait EventHandler<E: DomainEvent> {
    async fn handle(&self, event: E) -> anyhow::Result<()>;
}
