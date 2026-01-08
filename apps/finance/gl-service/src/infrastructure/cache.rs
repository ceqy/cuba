//! Cache Module for GL Service
//!
//! 查询缓存实现

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use rust_decimal::Decimal;
use tracing::{debug, info};

// ============================================================================
// Cache Entry
// ============================================================================

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

// ============================================================================
// Account Balance Cache
// ============================================================================

/// 科目余额缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BalanceCacheKey {
    pub company_code: String,
    pub gl_account: String,
    pub fiscal_year: i32,
    pub period: i32,
}

/// 科目余额缓存
pub struct AccountBalanceCache {
    cache: RwLock<HashMap<BalanceCacheKey, CacheEntry<Decimal>>>,
    default_ttl: Duration,
}

impl AccountBalanceCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// 获取缓存的余额
    pub fn get(&self, key: &BalanceCacheKey) -> Option<Decimal> {
        let cache = self.cache.read().ok()?;
        cache.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                debug!(key = ?key, "Cache hit for account balance");
                Some(entry.value)
            }
        })
    }

    /// 设置缓存
    pub fn set(&self, key: BalanceCacheKey, value: Decimal) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key, CacheEntry::new(value, self.default_ttl));
        }
    }

    /// 清除指定公司代码的缓存
    pub fn invalidate_company(&self, company_code: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|k, _| k.company_code != company_code);
            info!(company_code = %company_code, "Invalidated balance cache for company");
        }
    }

    /// 清除所有缓存
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
            info!("Cleared all balance cache");
        }
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) {
        if let Ok(mut cache) = self.cache.write() {
            let before = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            let removed = before - cache.len();
            if removed > 0 {
                debug!(removed = removed, "Cleaned up expired cache entries");
            }
        }
    }
}

// ============================================================================
// Cursor-based Pagination
// ============================================================================

/// 游标分页参数
#[derive(Debug, Clone)]
pub struct CursorPagination {
    pub cursor: Option<String>,
    pub limit: u32,
    pub direction: CursorDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorDirection {
    Forward,
    Backward,
}

impl Default for CursorPagination {
    fn default() -> Self {
        Self {
            cursor: None,
            limit: 50,
            direction: CursorDirection::Forward,
        }
    }
}

/// 游标分页结果
#[derive(Debug, Clone)]
pub struct CursorPagedResult<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub has_more: bool,
}

impl<T> CursorPagedResult<T> {
    pub fn empty() -> Self {
        Self {
            items: vec![],
            next_cursor: None,
            prev_cursor: None,
            has_more: false,
        }
    }
}

/// 游标编码/解码 (使用简单的格式)
pub mod cursor {
    /// 编码游标 (格式: id_timestamp)
    pub fn encode(id: &str, timestamp: i64) -> String {
        format!("{}_{}", id, timestamp)
    }

    /// 解码游标
    pub fn decode(cursor: &str) -> Option<(String, i64)> {
        let parts: Vec<&str> = cursor.rsplitn(2, '_').collect();
        if parts.len() == 2 {
            let timestamp = parts[0].parse().ok()?;
            let id = parts[1].to_string();
            Some((id, timestamp))
        } else {
            None
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_balance_cache() {
        let cache = AccountBalanceCache::new(60);
        let key = BalanceCacheKey {
            company_code: "1000".to_string(),
            gl_account: "1001000".to_string(),
            fiscal_year: 2026,
            period: 1,
        };

        assert!(cache.get(&key).is_none());

        cache.set(key.clone(), Decimal::from(10000));
        assert_eq!(cache.get(&key), Some(Decimal::from(10000)));

        cache.invalidate_company("1000");
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cursor_encoding() {
        let encoded = cursor::encode("abc123", 1704672000);
        let decoded = cursor::decode(&encoded);
        assert_eq!(decoded, Some(("abc123".to_string(), 1704672000)));
    }
}
