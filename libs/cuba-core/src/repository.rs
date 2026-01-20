use crate::domain::AggregateRoot;
use async_trait::async_trait;

#[async_trait]
pub trait Repository<T: AggregateRoot>: Send + Sync {
    type Id;
    async fn find_by_id(&self, id: &Self::Id) -> anyhow::Result<Option<T>>;
    async fn save(&self, entity: &T) -> anyhow::Result<()>;
}
