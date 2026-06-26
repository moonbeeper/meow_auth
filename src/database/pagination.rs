use crate::database::id::UlidId;

pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub next_id: Option<UlidId>,
    pub total_rows: Option<i64>,
}

impl<T: PaginatedId> PaginationResult<T> {
    // in the future, i would want to be able to fetch more than 20 items.
    // BUT, for now its better to hardcode it to 20
    pub fn new(data: Vec<T>, total_rows: Option<i64>) -> Self {
        let page_limit = data.len() > 20;
        let items: Vec<_> = data.into_iter().take(20).collect();

        let next_id = page_limit.then(|| {
            items
                .last()
                .expect("must exist, otherwise the hecking LENGTH is lying.")
                .paginated_id()
        });

        Self {
            items,
            next_id,
            total_rows,
        }
    }
}

pub trait PaginatedId {
    fn paginated_id(&self) -> UlidId;
}
