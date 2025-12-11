mod search;
mod search_context;

pub use search::*;
pub use search_context::SearchContext;

pub type SearchResult = Vec<usize>;

const MAX_SEARCH_RESULTS: usize = 200;
