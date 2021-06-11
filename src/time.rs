use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use num_traits::cast::FromPrimitive;

#[derive(Debug)]
pub enum TimeError {
    InvalidDay,
    InvalidMinute,
    InvalidTick,
    InvalidSubtick,

    InvalidUtcMonth,
    InvalidUtcDay,
    InvalidUtcHour,
    InvalidUtcMinute,
    InvalidUtcSecond,
    InvalidUtcNano
}

pub const SUBTICKS_PER_TICK: u32 = 1_000_000;
pub const TICKS_PER_MINUTE: u8 = 100;
pub const MINUTES_PER_DAY: u16 = 1000;

pub const TICKS_PER_DAY: u32 = MINUTES_PER_DAY as u32 * TICKS_PER_MINUTE as u32;

pub const DAYS_PER_COMMON_YEAR: u16 = 365;
pub const DAYS_PER_LEAP_YEAR: u16 = 366;

pub const MILLIS_PER_TICK: u16 = 864;
pub const MICROS_PER_TICK: u32 = 864_000;

#[derive(PartialEq, Clone, Debug)]
pub struct MetrumDate {
    year: i32,
    day: u16,
}
impl MetrumDate {
    pub fn new(year: i32, day: u16) -> Result<Self, TimeError> {
        let year_days = if is_leap_year(year) {DAYS_PER_LEAP_YEAR} else {DAYS_PER_COMMON_YEAR};
        if day >= year_days {
            Err(TimeError::InvalidDay)
        } else {
            Ok(Self { year, day })
        }
    }

    pub fn from_utc(year: i32, month: u8, day: u8) -> Result<Self, TimeError> {
        if month == 0 || month > 12 {
            return Err(TimeError::InvalidUtcMonth);
        }
        let chrono_month = chrono::Month::from_u32(month as u32).unwrap();
        if day > days_in_month(chrono_month, year) {
            return Err(TimeError::InvalidUtcDay);
        }
        Ok(Self {
            year,
            day: year_day(year, chrono_month, day)
        })
    }
}
impl std::fmt::Display for MetrumDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'{}", self.year, self.day)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct MetrumTime {
    minute: u16,
    tick: u8,
    subtick: u32
}
impl MetrumTime {
    pub fn new(minute: u16, tick: u8, subtick: u32) -> Result<Self, TimeError> {
        if minute > MINUTES_PER_DAY {
            return Err(TimeError::InvalidMinute);
        }
        if tick > TICKS_PER_MINUTE {
            return Err(TimeError::InvalidTick);
        }
        if subtick > SUBTICKS_PER_TICK {
            return Err(TimeError::InvalidSubtick);
        }
        Ok(Self { minute, tick, subtick })
    }

    pub fn from_utc(hour: u8, minute: u8, second: u8, nano: u32) -> Result<Self, TimeError> {
        if hour > 23 {
            return Err(TimeError::InvalidUtcHour);
        }
        if minute > 59 {
            return Err(TimeError::InvalidUtcMinute);
        }
        if second > 59 {
            return Err(TimeError::InvalidUtcSecond);
        }
        if nano > 999_999_999 {
            return Err(TimeError::InvalidUtcNano);
        }

        let day_micros: u64 = (hour as u64 * 3600 + minute as u64 * 60 + second as u64) * 1_000_000 + (nano as u64 / 1_000);
        let day_ticks: u32 = (day_micros / MICROS_PER_TICK as u64) as u32;
        let subtick: u32 = (day_micros % MICROS_PER_TICK as u64) as u32;

        Ok(
            Self { 
                minute: (day_ticks / TICKS_PER_MINUTE as u32) as u16,
                tick: (day_ticks % TICKS_PER_MINUTE as u32) as u8,
                subtick
            }
        )
    }

    pub fn minute(&self) -> u16 { self.minute }
    pub fn tick(&self) -> u8 { self.tick }
    pub fn subtick(&self) -> u32 { self.subtick }
    
}
impl std::fmt::Display for MetrumTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>3}:{:0>2}.{:0>6}", self.minute, self.tick, self.subtick)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct MetrumDateTime {
    date: MetrumDate,
    time: MetrumTime
}
impl MetrumDateTime {
    pub fn new(year: i32, day: u16, minute: u16, tick: u8, subtick: u32) -> Result<Self, TimeError> {
        Ok(Self {
            date: MetrumDate::new(year, day)?,
            time: MetrumTime::new(minute, tick, subtick)?
        })
    }

    pub fn from_timestamp(timestamp: i64) -> Self {
        let mut year: i32 = 2000;
        let mut timestamp_remaining = timestamp;

        let mut next_year_ticks = if is_leap_year(year + if timestamp_remaining < 0 {-1} else {0}) {366 * TICKS_PER_DAY as i64} else {365 * TICKS_PER_DAY as i64};

        while timestamp_remaining.abs() >= next_year_ticks {
            if timestamp_remaining < 0 {
                timestamp_remaining += next_year_ticks;
                year -= 1;
            } else {
                timestamp_remaining -= next_year_ticks;
                year += 1;
            }
            next_year_ticks = if is_leap_year(year + if timestamp_remaining < 0 {-1} else {0}) {366 * TICKS_PER_DAY as i64} else {365 * TICKS_PER_DAY as i64};
        }
        if timestamp_remaining >= 0 {
            let day = (timestamp_remaining / TICKS_PER_DAY as i64) as u16;
            let day_ticks = (timestamp_remaining % TICKS_PER_DAY as i64) as u32;
            let minute = (day_ticks / TICKS_PER_MINUTE as u32) as u16;
            let tick = (day_ticks % TICKS_PER_MINUTE as u32) as u8;

            Self {
                date: MetrumDate { year, day },
                time: MetrumTime { minute, tick, subtick: 0 }
            }
        } else {
            year -= 1;
            let day_count = if is_leap_year(year) {366} else {365};

            let mut day = (timestamp_remaining / TICKS_PER_DAY as i64 + day_count) as u16;
            let mut day_ticks = (timestamp_remaining % TICKS_PER_DAY as i64 + TICKS_PER_DAY as i64) as u32;
            if day_ticks != TICKS_PER_DAY {
                day -= 1;
            } else {
                day_ticks = 0;
            }
            let minute = (day_ticks / TICKS_PER_MINUTE as u32) as u16;
            let tick = (day_ticks % TICKS_PER_MINUTE as u32) as u8;

            Self {
                date: MetrumDate { year, day },
                time: MetrumTime { minute, tick, subtick: 0 }
            }
        }
    }

    pub fn from_utc(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8, nano: u32) -> Result<Self, TimeError> {
        Ok(Self {
            date: MetrumDate::from_utc(year, month, day)?,
            time: MetrumTime::from_utc(hour, minute, second, nano)?
        })
    }

    pub fn from_naive(naive: NaiveDateTime) -> Self {
        Self::from_utc(naive.year(), naive.month() as u8, naive.day() as u8, naive.hour() as u8, naive.minute() as u8, naive.second() as u8, naive.nanosecond()).unwrap()
    }

    pub fn now() -> Self {
        Self::from_naive(Utc::now().naive_utc())
    }

    pub fn year(&self) -> i32 {self.date.year}
    pub fn day(&self) -> u16 {self.date.day}
    pub fn minute(&self) -> u16 {self.time.minute}
    pub fn tick(&self) -> u8 {self.time.tick}
    pub fn subtick(&self) -> u32 {self.time.subtick}
    
    pub fn set_subtick(&mut self, subtick: u32) { self.time.subtick = subtick; }

    pub fn timestamp(&self) -> i64 {
        const YEAR_OFFSET: i32 = 2000;
        let offset_year = self.year() - YEAR_OFFSET;
        let mut timestamp_ticks = 0;

        if offset_year >= 0 {
            for current_year in 0..offset_year {
                let days_in_year = if is_leap_year(current_year + YEAR_OFFSET) { 366 } else { 365 };
                timestamp_ticks += days_in_year * 100_000;
            }
        } else if offset_year < 0 {
            for current_year in offset_year..0 {
                let days_in_year = if is_leap_year(current_year + YEAR_OFFSET) { 366 } else { 365 };
                timestamp_ticks -= days_in_year * 100_000;
            }
        }
        timestamp_ticks += self.day() as i64 * 100_000 + self.minute() as i64 * 100 + self.tick() as i64;

        timestamp_ticks 
    }
}


impl std::fmt::Display for MetrumDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}


pub fn year_day(year: i32, month: chrono::Month, day: u8) -> u16 {
    let mut year_day = day as u16 - 1; // day in the year starting at 0

    for previous_month in 1..month.number_from_month() {
        let month = chrono::Month::from_u32(previous_month).unwrap();
        year_day += days_in_month(month, year) as u16;
        
    }
    year_day
}

fn days_in_month(month: chrono::Month, year: i32) -> u8 {
    use chrono::Month::*;
    match month {
        January | March | May | July | August | October | December => 31,
        April | June | September | November => 30,
        February => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        
    }
}

fn is_leap_year(year: i32) -> bool {
    if year % 4 == 0 {
        if year % 100 == 0 {
            year % 400 == 0
        } else {
            true
        }
    } else {
        false
    }
}

#[test]
fn leap_years() {
    assert!(days_in_month(chrono::Month::February, 1980) == 29);
    assert!(days_in_month(chrono::Month::February, 2000) == 29);
    assert!(days_in_month(chrono::Month::February, 2020) == 29);

    assert!(days_in_month(chrono::Month::February, 1900) == 28);
    assert!(days_in_month(chrono::Month::February, 2014) == 28);
}

#[test]
fn constructors() {
    use chrono::TimeZone;
    let mut date_time = MetrumDateTime::from_naive(Utc.ymd(1970, 12, 24).naive_utc().and_time(chrono::NaiveTime::from_hms(12, 15, 17)));
    date_time.set_subtick(0);
    let mut same_date_time = MetrumDateTime::from_utc(1970, 12, 24, 12, 15, 17, 0).unwrap();
    same_date_time.set_subtick(0);
    let also_same_date_time = MetrumDateTime::new(date_time.year(), date_time.day(), date_time.minute(), date_time.tick(), date_time.subtick()).unwrap();
    assert_eq!(date_time, same_date_time);
    assert_eq!(date_time, also_same_date_time);
}

#[test]
fn timestamps() {
    let mut now = MetrumDateTime::now();
    now.set_subtick(0);
    let timestamp = now.timestamp();
    assert_eq!(MetrumDateTime::from_timestamp(timestamp), now);

    let mut moon_landing = MetrumDateTime::from_utc(1969, 7, 20, 20, 17, 40, 0).unwrap();
    moon_landing.set_subtick(0);
    assert_eq!(moon_landing, MetrumDateTime::from_timestamp(moon_landing.timestamp()));
}