use crate::storage::Transaction;

pub struct Notification {
    pub title: String,
    pub body: String,
}

pub trait Notifier: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, state: &mut Transaction) -> Option<Notification>;
}
