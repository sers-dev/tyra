#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Address {
    pub remote: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}