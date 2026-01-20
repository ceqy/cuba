//! Timestamp 转换工具
//! 统一处理 Proto Timestamp ↔ NaiveDate 的转换

use chrono::NaiveDate;
use prost_types::Timestamp;
use tonic::Status;

/// Proto Timestamp 转换为 NaiveDate
pub fn proto_to_naive_date(ts: &Timestamp) -> Result<NaiveDate, Status> {
    let datetime = chrono::DateTime::from_timestamp(ts.seconds, 0)
        .ok_or_else(|| Status::invalid_argument("Invalid timestamp"))?;
    Ok(datetime.naive_utc().date())
}

/// Option<Timestamp> 转换为 Option<NaiveDate>
pub fn proto_to_naive_date_opt(ts: Option<&Timestamp>) -> Option<NaiveDate> {
    ts.and_then(|t| chrono::DateTime::from_timestamp(t.seconds, 0))
        .map(|dt| dt.naive_utc().date())
}

/// NaiveDate 转换为 Proto Timestamp
pub fn naive_date_to_proto(date: NaiveDate) -> Timestamp {
    Timestamp {
        seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        nanos: 0,
    }
}

/// Option<NaiveDate> 转换为 Option<Timestamp>
pub fn naive_date_to_proto_opt(date: Option<NaiveDate>) -> Option<Timestamp> {
    date.map(naive_date_to_proto)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_proto_to_naive_date() {
        let ts = Timestamp {
            seconds: 1704067200, // 2024-01-01
            nanos: 0,
        };
        let date = proto_to_naive_date(&ts).unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn test_naive_date_to_proto() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let ts = naive_date_to_proto(date);
        assert_eq!(ts.seconds, 1704067200);
    }
}
