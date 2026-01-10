use async_trait::async_trait;
use cuba_errors::ServiceError;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publishes a message to a specific topic
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> Result<(), ServiceError>;
}

// Placeholder for future Kafka/Redpanda implementation
pub struct NoOpPublisher;

#[async_trait]
impl EventPublisher for NoOpPublisher {
    async fn publish(&self, _topic: &str, _key: &str, _payload: &[u8]) -> Result<(), ServiceError> {
        Ok(())
    }
}
