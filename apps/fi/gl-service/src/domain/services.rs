// Stub for domain services
pub trait JournalNumberGenerator {
    fn next_number(&self, company_code: &str, fiscal_year: i32) -> String;
}
