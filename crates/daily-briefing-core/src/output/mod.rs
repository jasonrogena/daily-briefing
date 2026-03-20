use crate::input::BoxError;

#[async_trait::async_trait]
pub trait Output: Send + Sync {
    fn name(&self) -> &str;
    async fn write(&self, content: &str) -> Result<(), BoxError>;
}
