use serde::Serialize;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationResponseDto<T> {
    page: i64,
    limit: i64,
    items: Vec<T>,
    done: bool,
}

impl<T> PaginationResponseDto<T> {
    pub fn from_query(query: PaginationQueryDto, items: Vec<T>) -> Self {
        let items_length = items.len();

        let items = items.into_iter()
            .take(query.get_limit_input() as usize)
            .collect();

        Self {
            page: query.get_page_input(),
            limit: query.get_limit_input(),
            items,
            done: items_length as i64 <= query.get_limit_input(),
        }
    }
}
