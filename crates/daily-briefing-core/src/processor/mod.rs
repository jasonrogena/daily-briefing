use crate::input::{BoxError, InputData};

#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    async fn process(&self, inputs: &[InputData]) -> Result<String, BoxError>;
}
