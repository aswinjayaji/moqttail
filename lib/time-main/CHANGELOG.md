# Changelog

All notable changes to the time project will be documented in this file.

The format is based on [Keep a Changelog]. This project adheres to [Semantic Versioning].

---

## 0.3.0 [_unreleased_]

### Added

- `datetime!` macro, which allows the construction of a statically verified `PrimitiveDateTime` or
  `OffsetDateTime`.
- `PrimitiveDateTime::replace_time`
- `PrimitiveDateTime::replace_date`
- `OffsetDateTime::replace_time`
- `OffsetDateTime::replace_date`
- `OffsetDateTime::replace_date_time`
- `OffsetDateTime::replace_offset`
- `#![no_alloc]` support
- `Date::to_iso_week_date`, replacing `Date::iso_year_week`
- `Date::MIN`
- `Date::MAX`
- `UtcOffset::from_hms`
- `UtcOffset::from_whole_seconds`
- `UtcOffset::as_hms`
- `UtcOffset::whole_hours`
- `UtcOffset::whole_minutes`
- `UtcOffset::minutes_past_hour`
- `UtcOffset::seconds_past_minute`
- `UtcOffset::is_utc`
- `UtcOffset::is_positive`
- `UtcOffset::is_negative`
- `OffsetDateTime::sunday_based_week`
- `OffsetDateTime::monday_based_week`
- `PrimitiveDateTime::to_calendar_date`
- `PrimitiveDateTime::to_ordinal_date`
- `PrimitiveDateTime::to_iso_week_date`
- `PrimitiveDateTime::to_julian_day`
- `OffsetDateTime::to_calendar_date`
- `OffsetDateTime::to_ordinal_date`
- `OffsetDateTime::to_iso_week_date`
- `OffsetDateTime::to_julian_day`
- `Time::as_hms`
- `Time::as_hms_milli`
- `Time::as_hms_micro`
- `Time::as_hms_nano`
- `PrimitiveDateTime::as_hms`
- `PrimitiveDateTime::as_hms_milli`
- `PrimitiveDateTime::as_hms_micro`
- `PrimitiveDateTime::as_hms_nano`
- `OffsetDateTime::to_hms`
- `OffsetDateTime::to_hms_milli`
- `OffsetDateTime::to_hms_micro`
- `OffsetDateTime::to_hms_nano`
- `Duration::saturating_add`
- `Duration::saturating_sub`
- `Duration::saturating_mul`
- `util::days_in_year_month`
- `Month`

### Changed

- The minimum supported Rust version is now 1.46.0.
- rand has been updated to 0.8.
- quickcheck has been updated to 1.0.
- Macros are placed behind the `macros` feature flag.
- Renamed
  - `OffsetDatetime::timestamp` → `OffsetDateTime::unix_timestamp`
  - `OffsetDatetime::timestamp_nanos` → `OffsetDateTime::unix_timestamp_nanos`
  - `Date::try_from_ymd` → `Date::from_calendar_date`
  - `Date::try_from_yo` → `Date::from_ordinal_date`
  - `Date::try_from_iso_ywd` → `Date::from_iso_week_date`
  - `Date::as_ymd` → `Date::to_calendar_date`
  - `Date::as_yo` → `Date::to_ordinal_date`
  - `Date::try_with_hms` → `Date::with_hms`
  - `Date::try_with_hms_milli` → `Date::with_hms_milli`
  - `Date::try_with_hms_micro` → `Date::with_hms_micro`
  - `Date::try_with_hms_nano` → `Date::with_hms_nano`
  - `Time::try_from_hms` → `Time::from_hms`
  - `Time::try_from_hms_milli` → `Time::from_hms_milli`
  - `Time::try_from_hms_micro` → `Time::from_hms_micro`
  - `Time::try_from_hms_nano` → `Time::from_hms_nano`
  - `UtcOffset::try_local_offset_at` → `UtcOffset::local_offset_at`
  - `UtcOffset::as_seconds` → `UtcOffset::whole_seconds`
  - `OffsetDateTime::try_now_local` → `OffsetDateTime::now_local`
  - `Date::week` → `Date::iso_week`
  - `PrimitiveDateTime::week` → `PrimitiveDateTime::iso_week`
  - `OffsetDateTime::week` → `OffsetDateTime::iso_week`
  - `Date::julian_day` → `Date::to_julian_day`
  - Macros have been moved to the `macros` module, but are otherwise named the same.
  - All `Duration` unit values, as well as the minimum and maximum, are now associated constants.
  - `OffsetDateTime::unix_epoch()` → `OffsetDateTime::UNIX_EPOCH`
  - `Time::midnight()` → `Time::MIDNIGHT`
- Now `const fn` (on at least newer compilers)
  - `Date::weekday`
  - `Date::next_day`
  - `Date::previous_day`
  - `PrimitiveDateTime::assume_offset`
  - `PrimitiveDateTime::weekday`
  - `Duration::checked_add`
  - `Duration::checked_sub`
  - `Duration::checked_mul`
  - `OffsetDateTime::from_unix_timestamp`
  - `OffsetDateTime::from_unix_timestamp_nanos`
  - `OffsetDateTime::date`
  - `OffsetDateTime::time`
  - `OffsetDateTime::year`
  - `OffsetDateTime::month`
  - `OffsetDateTime::day`
  - `OffsetDateTime::ordinal`
  - `OffsetDateTime::to_iso_week_date`
  - `OffsetDateTime::week`
  - `OffsetDateTime::weekday`
  - `OffsetDateTime::hour`
  - `OffsetDateTime::minute`
  - `OffsetDateTime::second`
  - `OffsetDateTime::millisecond`
  - `OffsetDateTime::microsecond`
  - `OffsetDateTime::nanosecond`
  - `OffsetDateTime::unix_timestamp`
  - `OffsetDateTime::unix_timestamp_nanos`
- Some variants of `Error` no longer contain an inner item. This is because the item is already
  guaranteed to be a zero-sized struct.
- The following functions now return a `Result`:
  - `Date::from_julian_day`
  - `OffsetDateTime::from_unix_timestamp`
  - `OffsetDateTime::from_unix_timestamp_nanos`
- The following functions now return an `Option`:
  - `Date::next_day`
  - `Date::previous_day`
- The range of valid years has changed. By default, it is ±9999. When using the `large-dates`
  feature, this is increased to ±999,999. Enabling the feature has performance implications and
  introduces ambiguities when parsing.
- The following are now gated under the `local-offset` feature:
  - `UtcOffset::local_offset_at`
  - `OffsetDateTime::now_local`
- `Instant` is now guaranteed to be represented as a tuple struct containing a `std::time::Instant`.
- Macros now simulate `const` blocks, guaranteeing that the value is statically generated.
- `Date::to_julian_day` now returns an `i32` (was `i64`).
- `Date::from_julian_day` now accepts an `i32` (was `i64`).
- Extension traits are only implemented for some types and are now sealed. As they are intended to
  be used with value literals, the breakage caused by this should be minimal.
- The new `Month` enum is used instead of numerical values where appropriate.

### Removed

- v0.1 APIs, previously behind an enabled-by-default feature flag
  - `PreciseTime`
  - `SteadyTime`
  - `precise_time_ns`
  - `precise_time_s`
  - `Instant::to`
  - `Duration::num_weeks`
  - `Duration::num_days`
  - `Duration::num_hours`
  - `Duration::num_minutes`
  - `Duration::num_seconds`
  - `Duration::num_milliseconds`
  - `Duration::num_microseconds`
  - `Duration::num_nanoseconds`
  - `Duration::span`
  - `Duration::from_std`
  - `Duration::to_std`
- Panicking APIs, previously behind a non-default feature flag
  - `Date::from_ymd`
  - `Date::from_yo`
  - `Date::from_iso_ywd`
  - `Date::with_hms`
  - `Date::with_hms_milli`
  - `Date::with_hms_micro`
  - `Date::with_hms_nano`
  - `Time::from_hms`
  - `Time::from_hms_milli`
  - `Time::from_hms_micro`
  - `Time::from_hms_nano`
- APIs that assumed an offset of UTC, previously enabled unconditionally
  - `Date::today`
  - `Time::now`
  - `PrimitiveDateTime::now`
  - `PrimitiveDateTime::unix_epoch`
  - `PrimitiveDateTime::from_unix_timestamp`
  - `PrimitiveDateTime::timestamp`
  - `OffsetDateTime::now`
  - `impl Sub<SystemTime> for PrimitiveDateTime`
  - `impl Sub<PrimitiveDateTime> for SystemTime`
  - `impl PartialEq<SystemTime> for PrimitiveDateTime`
  - `impl PartialEq<PrimitiveDateTime> for SystemTime`
  - `impl PartialOrd<SystemTime> for PrimitiveDateTime`
  - `impl PartialOrd<PrimitiveDateTime> for SystemTime`
  - `impl From<SystemTime> for PrimitiveDateTime`
  - `impl From<PrimitiveDateTime> for SystemTime`
  - `UtcOffset::local_offset_at` — assumed UTC if unable to determine local offset
  - `OffsetDateTime::now_local` — assumed UTC if unable to determine local offset
- Other APIs deprecated during the course of 0.2, previously enabled unconditionally
  - `Duration::sign`
  - `PrimitiveDateTime::using_offset`
  - `Sign`
- Re-exports of APIs moved during the course of 0.2
  - `days_in_year`
  - `is_leap_year`
  - `validate_format_string`
  - `weeks_in_year`
  - `ComponentRangeError`
  - `ConversionRangeError`
  - `IndeterminateOffsetError`
  - `ParseError`
  - `NumericalDuration`
  - `NumericalStdDuration`
  - `NumericalStdDurationShort`
- Lazy formatting, which was unidiomatic as a failure would have returned `fmt::Error`, indicating
  an error unrelated to the time crate.
  - `Time::lazy_format`
  - `Date::lazy_format`
  - `UtcOffset::lazy_format`
  - `PrimitiveDateTime::lazy_format`
  - `OffsetDateTime::lazy_format`
- Support for stdweb has been removed, as the crate is unmaintained.
- The `prelude` module has been removed in its entirety.
- `Date::iso_year_week`, in favor of `Date::to_iso_week_date`
- `PrimitiveDateTime::iso_year_week`
- `OffsetDateTime::iso_year_week`
- `UtcOffset::east_hours`
- `UtcOffset::west_hours`
- `UtcOffset::hours`
- `UtcOffset::east_minutes`
- `UtcOffset::west_minutes`
- `UtcOffset::minutes`
- `UtcOffset::east_seconds`
- `UtcOffset::west_seconds`
- `UtcOffset::seconds`
- `Date::month_day`
- `PrimitiveDateTime::month_day`
- `OffsetDateTime::month_day`
- `Weekday::iso_weekday_number` (identical to `Weekday::number_from_monday`)
- `ext::NumericalStdDurationShort`

## 0.2.22 [2020-09-25]

### Fixed

- Solaris & Illumos now successfully build.
- `Duration::new` could previously result in an inconsistent internal state. This led to some odd
  situations where a `Duration` could be both positive and negative. This has been fixed such that
  the internal state maintains its invariants.

## 0.2.21 [2020-09-20]

### Changed

- Implementation details of some error types have been exposed. This means that data about a
  component being out of range can be directly obtained, while an invalid offset or conversion error
  is guaranteed to be a zero-sized type.
- The following functions are `const fn` on rustc ≥ 1.46:
  - `Date::try_from_iso_ywd`
  - `Date::iso_year_week`
  - `Date::week`
  - `Date::sunday_based_week`
  - `Date::monday_based_week`
  - `Date::try_with_hms`
  - `Date::try_with_hms_milli`
  - `Date::try_with_hms_micro`
  - `Date::try_with_hms_nano`
  - `PrimitiveDateTime::iso_year_week`
  - `PrimitiveDateTime::week`
  - `PrimitiveDateTime::sunday_based_week`
  - `PrimitiveDateTime::monday_based_week`
  - `util::weeks_in_year`

## 0.2.20 [2020-09-16]

### Added

- `OffsetDateTime::timestamp_nanos`
- `OffsetDateTime::from_unix_timestamp_nanos`

### Fixed

A bug with far-reaching consequences has been fixed. See #276 for complete details, but the gist is
that the constructing a `Date` from a valid Julian day may result in an invalid value or even panic.
As a consequence of implementation details, this affects nearly all arithmetic with `Date`s (and as
a result also `PrimitiveDateTime`s and `OffsetDateTime`s).

### Improvements

- Document how to construct an `OffsetDateTime` from a timestamp-nanosecond pair

## 0.2.19 [2020-09-12]

### Fixed

- The build script now declares a dependency on the `COMPILING_UNDER_CARGO_WEB` environment
  variable.
- Parsing the `%D` specifier no longer requires padding on the month. Previously,
  `Err(InvalidMonth)` was incorrectly returned.
- A `std::time::Duration` that is larger than `time::Duration::max_value()` now correctly returns
  `Ordering::Greater` when compared.
- Multiplying and assigning an integer by `Sign::Zero` now sets the integer to be zero. This
  previously left the integer unmodified.

## 0.2.18 [2020-09-08]

### Changed

- The following functions are `const fn` on rustc ≥ 1.46:
  - `Date::try_from_ymd`
  - `Date::try_from_yo`
  - `Time::try_from_hms`
  - `Time::try_from_hms_milli`
  - `Time::try_from_hms_micro`
  - `Time::try_from_hms_nano`
- An `error` module has been created where all existing error types are contained. The `Error`
  suffix has been dropped from these types.
- An `ext` module has been created where extension traits are contained.
- A `util` module has been created where utility functions are contained.
- `error::ComponentRange` now implements `Copy`.

For back-compatibility, all items that were moved to newly-contained modules have been re-exported
from their previous locations (and in the case of the `error` module, with their previous name).

### Fixes

Parsing `format::Rfc3339` now correctly handles the UTC offset (#274).

## 0.2.17 [2020-09-01]

### Changed

The following functions are `const fn` on rustc ≥ 1.46:

- `Date::year`
- `Date::month`
- `Date::day`
- `Date::month_day`
- `Date::ordinal`
- `Date::as_ymd`
- `Date::as_yo`
- `Date::julian_day`
- `Duration::checked_div`
- `PrimitiveDateTime::year`
- `PrimitiveDateTime::month`
- `PrimitiveDateTime::day`
- `PrimitiveDateTime::month_day`
- `PrimitiveDateTime::ordinal`
- `Weekday::previous`
- `Weekday::next`

### Improvements

- `size_of::<Date>()` has been reduced from 8 to 4. As a consequence,
  `size_of::<PrimitiveDatetime>()` went from 16 to 12 and `size_of::<OffsetDateTime>()` from 20
  to 16. This change also results in a performance improvement of approximately 30% on the
  `Date::year` and `Date::ordinal` methods.
- `cfg-if` has been removed as a dependency.

### Fixed

- `cfg` flags passed to rustc will no longer collide with other crates (at least unless they're
  doing something very stupid).
- The crate will successfully compile with any combination of feature flags. Previously, some
  combinations would fail.

## 0.2.16 [2020-05-12]

### Added

`OffsetDateTime`s can now be represented as Unix timestamps with serde. To do this, you can use the
`time::serde::timestamp` and `time::serde::timestamp::option` modules.

## 0.2.15 [2020-05-04]

### Fixed

`cargo-web` support works, and is now explicitly checked in CI. A previous change was made that made
a method call ambiguous.

## 0.2.14 [2020-05-02]

### Fixed

Adding/subtracting a `core::time::Duration` now correctly takes subsecond values into account. This
also affects `PrimitiveDateTime` and `OffsetDateTime`.

## 0.2.13 [2020-05-01]

### Fixed

Panicking APIs are re-exposed.

## 0.2.12 [2020-04-30]

### Fixed

Subtracting `Instant`s can correctly result in a negative duration, rather than resulting in the
absolute value of it.

## 0.2.11 [2020-04-27]

### Added

- `OffsetDateTime::now_utc`

### Deprecated

- `OffsetDateTime::now` due to the offset not being clear from the method name alone.

### Fixed

`Date`s are now uniformly random when using the `rand` crate. Previously, both the year and day
within the year were uniform, but this meant that any given day in a leap year was slightly less
likely to be chosen than a day in a non-leap year.

### Changed

- MSRV is lowered to 1.32.0.

## 0.2.10 [2020-04-19]

### Added

- Support for formatting and parsing `OffsetDateTime`s as RFC3339.
- Lazy formatting. To avoid exposing implementation details, we're just returning `impl Display`, rather than a concrete type.
- Add support for Illumos.

### Fixed

- Deprecated APIs from time v0.1 are public again. They were previously hidden by accident in 0.2.9.

## 0.2.9 [2020-03-13]

### Fixed

`cfg-if` now has a mandatory minimum of 0.1.10, rather than just 0.1. This is because compilation
fails when using 0.1.9.

## 0.2.8 [2020-03-12]

### Added

- `cargo_web` support has been added for getting a local offset. A general catch-all defaulting to
  UTC has also been added.
- `Error::source` has been implemented for the wrapper `time::Error`.
- `UtcOffset::try_local_offset`, `UtcOffset::try_current_local_offset`,
  `OffsetDateTime::try_now_local()` provide fallible alternatives when the default of UTC is not
  desired. To facilitate this change, `IndeterminateOffsetError` has been added.
- Support for parsing and formatting subsecond nanoseconds.

### Changed

- `#[non_exhaustive]` is simulated on compilers prior to 1.40.0.

## 0.2.7 [2020-02-22]

### Added

- `Display` has been implemented for `Date`, `OffsetDateTime`, `PrimitiveDateTime`, `Time`,
  `UtcOffset`, and `Weekday`.
- `Hash` is now derived for `Duration`.
- `SystemTime` can be converted to and from `OffsetDateTime`. The following trait implementations
  have been made for interoperability:
  - `impl Sub<SystemTime> for OffsetDateTime`
  - `impl Sub<OffsetDateTime> for SystemTime`
  - `impl PartialEq<SystemTime> for OffsetDateTime`
  - `impl PartialEq<OffsetDateTime> for SystemTime`
  - `impl PartialOrd<SystemTime> for OffsetDateTime`
  - `impl PartialOrd<OffsetDateTime> for SystemTime`
  - `impl From<SystemTime> for OffsetDateTime`
  - `impl From<OffsetDateTime> for SystemTime`
- All structs now `impl Duration<T> for Standard`, allowing usage with the `rand` crate. This is
  gated behind the `rand` feature flag.
- Documentation can now be built on stable. Some annotations will be missing if you do this.
- `NumericalDuration` has been implemented for `f32` and `f64`. `NumericalStdDuration` and
  `NumericalStdDurationShort` have been implemented for `f64` only.
- `UtcOffset::local_offset_at(OffsetDateTime)`, which will obtain the system's local offset at the
  provided moment in time.
  - `OffsetDateTime::now_local()` is equivalent to calling
    `OffsetDateTime::now().to_offset(UtcOffset::local_offset_at(OffsetDateTime::now()))` (but more
    efficient).
  - `UtcOffset::current_local_offset()` will return the equivalent of
    `OffsetDateTime::now_local().offset()`.

### Changed

- All formatting and parsing methods now accept `impl AsRef<str>` as parameters, rather than just
  `&str`. `time::validate_format_string` does this as well.
- The requirement of a `Date` being between the years -100,000 and +100,000 (inclusive) is now
  strictly enforced.
- Overflow checks for `Duration` are now enabled by default. This behavior is the identical to what
  the standard library does.
- The `time`, `date`, and `offset` macros have been added to the prelude.

### Deprecated

- `Sign` has been deprecated in its entirety, along with `Duration::sign`.

  To obtain the sign of a `Duration`, you can use the `Duration::is_positive`,
  `Duration::is_negative`, and `Duration::is_zero` methods.

- A number of functions and trait implementations that implicitly assumed a timezone (generally UTC)
  have been deprecated. These are:
  - `Date::today`
  - `Time::now`
  - `PrimitiveDateTime::now`
  - `PrimitiveDateTime::unix_epoch`
  - `PrimitiveDateTime::from_unix_timestamp`
  - `PrimitiveDateTime::timestamp`
  - `impl Sub<SystemTime> for PrimitiveDateTime`
  - `impl Sub<PrimitiveDateTime> for SystemTime`
  - `impl PartialEq<SystemTime> for PrimitiveDateTime`
  - `impl PartialEq<PrimitiveDateTime> for SystemTime>`
  - `impl PartialOrd<SystemTime> for PrimitiveDateTime`
  - `impl PartialOrd<PrimitiveDateTime> for SystemTime>`
  - `impl From<SystemTime> for PrimitiveDateTime`
  - `impl From<PrimitiveDateTime> for SystemTime`

### Fixed

- Avoid panics when parsing an empty string (#215).
- The nanoseconds component of a `Duration` is now always in range. Previously, it was possible (via
  addition and/or subtraction) to obtain a value that was not internally consistent.
- `Time::parse` erroneously returned an `InvalidMinute` error when it was actually the second that
  was invalid.
- `Date::parse("0000-01-01", "%Y-%m-%d")` incorrectly returned an `Err` (#221).

## Pre-0.2.7

Prior to v0.2.7, changes were listed in GitHub releases.

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
