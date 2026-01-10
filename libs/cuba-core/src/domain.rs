use async_trait::async_trait;

/// Marker trait for Aggregate Roots
pub trait AggregateRoot: Send + Sync {}

/// Marker trait for Entities
pub trait Entity: Send + Sync {
    type Id;
    fn id(&self) -> &Self::Id;
}

/// Marker trait for Value Objects
pub trait ValueObject: Clone + PartialEq + Send + Sync {}
