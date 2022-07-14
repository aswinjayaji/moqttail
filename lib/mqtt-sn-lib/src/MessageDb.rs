
#[derive(Debug, Clone)]
pub struct MessageDb {
    pub db: sled::Db,
    pub name: String,
    pub old_value: String,
}
