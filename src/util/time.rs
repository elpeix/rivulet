use crate::i18n::Lang;
use time::OffsetDateTime;

pub fn now_timestamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

pub fn format_timestamp(timestamp: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(timestamp) {
        Ok(dt) => {
            let date = dt.date();
            let time = dt.time();
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                date.year(),
                date.month() as u8,
                date.day(),
                time.hour(),
                time.minute()
            )
        }
        Err(_) => timestamp.to_string(),
    }
}

pub fn format_timestamp_short(timestamp: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(timestamp) {
        Ok(dt) => {
            let date = dt.date();
            format!(
                "{:04}-{:02}-{:02}",
                date.year(),
                date.month() as u8,
                date.day()
            )
        }
        Err(_) => timestamp.to_string(),
    }
}

pub fn format_timestamp_relative(timestamp: i64, lang: &Lang) -> String {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let diff = now - timestamp;

    if diff < 0 {
        return format_timestamp_short(timestamp);
    }

    let seconds = diff;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;

    if seconds < 60 {
        lang.now.to_string()
    } else if minutes < 60 {
        lang.minutes_ago(minutes)
    } else if hours < 24 {
        lang.hours_ago(hours)
    } else if days == 1 {
        lang.yesterday.to_string()
    } else if days < 7 {
        lang.days_ago(days)
    } else if weeks < 4 {
        lang.weeks_ago(weeks)
    } else {
        format_timestamp_short(timestamp)
    }
}
