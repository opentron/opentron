//! Helpers for transaction.

use chrono::Utc;

pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

