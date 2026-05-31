use crate::storage::Storage;

pub struct Notification {
    pub title: String,
    pub body: String,
}

pub trait Notifier: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, storage: &mut Storage) -> Option<Notification>;
}
