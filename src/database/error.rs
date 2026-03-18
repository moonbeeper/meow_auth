#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("something went wrong with sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
