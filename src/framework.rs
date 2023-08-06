#[async_trait::async_trait]
pub trait ChainLink {
    type TInput;
    type TOutput;

    async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<Self::TInput>>);
    async fn push_raw(&self, input: Self::TInput);
    async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<Self::TInput>>);
    async fn push_raw_if_empty(&self, input: Self::TInput);
    async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<Self::TOutput>>>;
    async fn process(&self) -> bool;
}