#[derive(Debug, FromForm)]
pub struct PaginationQueryDto {
    #[field(default = 1, validate = range(1..))]
    pub page: i64,

    #[field(default = 50, validate = range(0..))]
    pub limit: i64,
}

impl PaginationQueryDto {
    pub fn get_page_input(&self) -> i64 {
        self.page
    }

    pub fn get_limit_input(&self) -> i64 {
        self.limit
    }

    pub fn get_offset(&self) -> i64 {
        (self.get_page_input() - 1) * self.get_limit_input()
    }

    pub fn get_limit(&self) -> i64 {
        self.get_limit_input() + 1
    }
}
