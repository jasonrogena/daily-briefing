pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct InputData {
    pub source: String,
    pub content: String,
}

#[async_trait::async_trait]
pub trait Input: Send + Sync {
    fn name(&self) -> &str;
    async fn collect(&self) -> Result<InputData, BoxError>;
}
