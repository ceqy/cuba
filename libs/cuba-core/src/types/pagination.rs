use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub ascending: bool,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: None,
            ascending: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total_items: u64,
    pub current_page: u32,
    pub total_pages: u32,
}

impl<T> PageResponse<T> {
    pub fn new(items: Vec<T>, total_items: u64, page: u32, page_size: u32) -> Self {
        let total_pages = (total_items as f64 / page_size as f64).ceil() as u32;
        Self {
            items,
            total_items,
            current_page: page,
            total_pages,
        }
    }
}
