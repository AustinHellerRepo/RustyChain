use tokio::sync::Mutex;


pub struct Queue<T> {
    items: Mutex<Vec<T>>
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Queue {
            items: Mutex::new(Vec::new())
        }
    }
}

impl<T> Queue<T> {
    pub async fn push(&self, item: T) {
        let mut locked_items = self.items.lock().await;
        locked_items.push(item);
    }
    pub async fn push_if_empty(&self, item: T) {
        let mut locked_items = self.items.lock().await;
        if locked_items.is_empty() {
            locked_items.push(item);
        }
    }
    pub async fn try_pop(&self) -> Option<T> {
        let popped_item: Option<T>;
        {
            let mut locked_items = self.items.lock().await;
            popped_item = if locked_items.len() == 0 {
                None
            }
            else {
                Some(locked_items.remove(0usize))
            };
        }
        return popped_item;
    }
}