use async_trait::async_trait;
use cuba_cqrs::DomainEvent;

#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish a domain event to the bus
    async fn publish<E: DomainEvent>(&self, event: E) -> anyhow::Result<()>;
}

/// A simple memory-based event bus for testing
pub struct MemoryEventBus;

#[async_trait]
impl EventBus for MemoryEventBus {
    async fn publish<E: DomainEvent>(&self, event: E) -> anyhow::Result<()> {
        println!("MemoryEventBus published event: {:?}", event);
        Ok(())
    }
}
