use serde_test::{assert_de_tokens_error, assert_tokens, Compact, Configure, Readable, Token};
use time::macros::{date, datetime, offset, time};
use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};

#[test]
fn time() {
    assert_tokens(
        &Time::MIDNIGHT.compact(),
        &[
            Token::Tuple { len: 4 },
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &time!(23:58:59.123_456_789).compact(),
        &[
            Token::Tuple { len: 4 },
            Token::U8(23),
            Token::U8(58),
            Token::U8(59),
            Token::U32(123_456_789),
            Token::TupleEnd,
        ],
    );
    assert_de_tokens_error::<Compact<Time>>(
        &[
            Token::Tuple { len: 4 },
            Token::U8(24),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `24`, expected a value in the range 0..=23",
    );

    assert_tokens(
        &Time::MIDNIGHT.readable(),
        &[Token::BorrowedStr("00:00:00.0")],
    );
    assert_tokens(
        &time!(23:58:59.123_456_789).readable(),
        &[Token::BorrowedStr("23:58:59.123456789")],
    );
    assert_de_tokens_error::<Readable<Time>>(
        &[Token::BorrowedStr("24:00:00.0")],
        "invalid value: integer `24`, expected a value in the range 0..=23",
    );
    assert_de_tokens_error::<Readable<Time>>(
        &[Token::BorrowedStr("24-00:00.0")],
        "invalid value: literal, expected valid format",
    );
    assert_de_tokens_error::<Readable<Time>>(
        &[Token::BorrowedStr("0:00:00.0")],
        "invalid value: hour, expected valid hour",
    );
    assert_de_tokens_error::<Readable<Time>>(
        &[Token::BorrowedStr("00:00:00.0x")],
        "invalid value: literal, expected no extraneous characters",
    );
    assert_de_tokens_error::<Readable<Time>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<Time>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a tuple of size 4",
    );
}

#[test]
fn date() {
    assert_tokens(
        &date!(-9999 - 001).compact(),
        &[
            Token::Tuple { len: 2 },
            Token::I32(-9999),
            Token::U16(1),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &date!(+9999-365).compact(),
        &[
            Token::Tuple { len: 2 },
            Token::I32(9999),
            Token::U16(365),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &date!(-9999 - 001).readable(),
        &[Token::BorrowedStr("-9999-01-01")],
    );
    assert_tokens(
        &date!(+9999-365).readable(),
        &[Token::BorrowedStr("9999-12-31")],
    );
    assert_de_tokens_error::<Readable<Date>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<Date>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a tuple of size 2",
    );
}

#[test]
fn primitive_date_time() {
    assert_tokens(
        &datetime!(-9999-001 0:00).compact(),
        &[
            Token::Tuple { len: 6 },
            Token::I32(-9999),
            Token::U16(1),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &datetime!(+9999-365 23:58:59.123_456_789).compact(),
        &[
            Token::Tuple { len: 6 },
            Token::I32(9999),
            Token::U16(365),
            Token::U8(23),
            Token::U8(58),
            Token::U8(59),
            Token::U32(123_456_789),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &datetime!(-9999-001 0:00).readable(),
        &[Token::BorrowedStr("-9999-01-01 00:00:00.0")],
    );
    assert_tokens(
        &datetime!(+9999-365 23:58:59.123_456_789).readable(),
        &[Token::BorrowedStr("9999-12-31 23:58:59.123456789")],
    );
    assert_de_tokens_error::<Readable<PrimitiveDateTime>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<PrimitiveDateTime>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a tuple of size 6",
    );
    assert_de_tokens_error::<Compact<PrimitiveDateTime>>(
        &[
            Token::Tuple { len: 6 },
            Token::I32(2021),
            Token::U16(366),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `366`, expected a value in the range 1..=365",
    );
    assert_de_tokens_error::<Compact<PrimitiveDateTime>>(
        &[
            Token::Tuple { len: 6 },
            Token::I32(2021),
            Token::U16(1),
            Token::U8(24),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `24`, expected a value in the range 0..=23",
    );
}

#[test]
fn offset_date_time() {
    assert_tokens(
        &datetime!(-9999-001 0:00 UTC)
            .to_offset(offset!(+23:58:59))
            .compact(),
        &[
            Token::Tuple { len: 9 },
            Token::I32(-9999),
            Token::U16(1),
            Token::U8(23),
            Token::U8(58),
            Token::U8(59),
            Token::U32(0),
            Token::I8(23),
            Token::I8(58),
            Token::I8(59),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &datetime!(+9999-365 23:58:59.123_456_789 UTC)
            .to_offset(offset!(-23:58:59))
            .compact(),
        &[
            Token::Tuple { len: 9 },
            Token::I32(9999),
            Token::U16(365),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U32(123_456_789),
            Token::I8(-23),
            Token::I8(-58),
            Token::I8(-59),
            Token::TupleEnd,
        ],
    );

    assert_tokens(
        &datetime!(-9999-001 0:00 UTC)
            .to_offset(offset!(+23:58:59))
            .readable(),
        &[Token::BorrowedStr("-9999-01-01 23:58:59.0 +23:58:59")],
    );
    assert_tokens(
        &datetime!(+9999-365 23:58:59.123_456_789 UTC)
            .to_offset(offset!(-23:58:59))
            .readable(),
        &[Token::BorrowedStr(
            "9999-12-31 00:00:00.123456789 -23:58:59",
        )],
    );
    assert_de_tokens_error::<Readable<OffsetDateTime>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<OffsetDateTime>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a tuple of size 9",
    );
    assert_de_tokens_error::<Compact<OffsetDateTime>>(
        &[
            Token::Tuple { len: 9 },
            Token::I32(2021),
            Token::U16(366),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::I8(0),
            Token::I8(0),
            Token::I8(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `366`, expected a value in the range 1..=365",
    );
    assert_de_tokens_error::<Compact<OffsetDateTime>>(
        &[
            Token::Tuple { len: 9 },
            Token::I32(2021),
            Token::U16(1),
            Token::U8(24),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::I8(0),
            Token::I8(0),
            Token::I8(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `24`, expected a value in the range 0..=23",
    );
    assert_de_tokens_error::<Compact<OffsetDateTime>>(
        &[
            Token::Tuple { len: 9 },
            Token::I32(2021),
            Token::U16(1),
            Token::U8(0),
            Token::U8(0),
            Token::U8(0),
            Token::U32(0),
            Token::I8(24),
            Token::I8(0),
            Token::I8(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `24`, expected a value in the range -23..=23",
    );
}

#[test]
fn utc_offset() {
    assert_tokens(
        &offset!(-23:58:59).compact(),
        &[
            Token::Tuple { len: 3 },
            Token::I8(-23),
            Token::I8(-58),
            Token::I8(-59),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &offset!(+23:58:59).compact(),
        &[
            Token::Tuple { len: 3 },
            Token::I8(23),
            Token::I8(58),
            Token::I8(59),
            Token::TupleEnd,
        ],
    );

    assert_tokens(
        &offset!(-23:58:59).readable(),
        &[Token::BorrowedStr("-23:58:59")],
    );
    assert_tokens(
        &offset!(+23:58:59).readable(),
        &[Token::BorrowedStr("+23:58:59")],
    );
    assert_de_tokens_error::<Readable<UtcOffset>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<UtcOffset>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a tuple of size 3",
    );
    assert_de_tokens_error::<Compact<UtcOffset>>(
        &[
            Token::Tuple { len: 3 },
            Token::I8(24),
            Token::I8(0),
            Token::I8(0),
            Token::TupleEnd,
        ],
        "invalid value: integer `24`, expected a value in the range -23..=23",
    );
}

#[test]
fn duration() {
    assert_tokens(
        &Duration::MIN.compact(),
        &[
            Token::Tuple { len: 2 },
            Token::I64(i64::MIN),
            Token::I32(-999_999_999),
            Token::TupleEnd,
        ],
    );
    assert_tokens(
        &Duration::MAX.compact(),
        &[
            Token::Tuple { len: 2 },
            Token::I64(i64::MAX),
            Token::I32(999_999_999),
            Token::TupleEnd,
        ],
    );

    assert_tokens(
        &Duration::MIN.readable(),
        &[Token::BorrowedStr("-9223372036854775808.999999999")],
    );
    assert_tokens(
        &Duration::MAX.readable(),
        &[Token::BorrowedStr("9223372036854775807.999999999")],
    );
    assert_tokens(
        &Duration::ZERO.readable(),
        &[Token::BorrowedStr("0.000000000")],
    );
    assert_de_tokens_error::<Readable<Duration>>(
        &[Token::BorrowedStr("x")],
        r#"invalid value: string "x", expected a decimal point"#,
    );
    assert_de_tokens_error::<Readable<Duration>>(
        &[Token::BorrowedStr("x.0")],
        r#"invalid value: string "x", expected a number"#,
    );
    assert_de_tokens_error::<Readable<Duration>>(
        &[Token::BorrowedStr("0.x")],
        r#"invalid value: string "x", expected a number"#,
    );
    assert_de_tokens_error::<Readable<Duration>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<Duration>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a tuple of size 2",
    );
}

#[test]
fn weekday() {
    assert_tokens(&Weekday::Monday.compact(), &[Token::U8(1)]);
    assert_tokens(&Weekday::Tuesday.compact(), &[Token::U8(2)]);
    assert_tokens(&Weekday::Wednesday.compact(), &[Token::U8(3)]);
    assert_tokens(&Weekday::Thursday.compact(), &[Token::U8(4)]);
    assert_tokens(&Weekday::Friday.compact(), &[Token::U8(5)]);
    assert_tokens(&Weekday::Saturday.compact(), &[Token::U8(6)]);
    assert_tokens(&Weekday::Sunday.compact(), &[Token::U8(7)]);
    assert_de_tokens_error::<Compact<Weekday>>(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a value in the range 1..=7",
    );

    assert_tokens(&Weekday::Monday.readable(), &[Token::BorrowedStr("Monday")]);
    assert_tokens(
        &Weekday::Tuesday.readable(),
        &[Token::BorrowedStr("Tuesday")],
    );
    assert_tokens(
        &Weekday::Wednesday.readable(),
        &[Token::BorrowedStr("Wednesday")],
    );
    assert_tokens(
        &Weekday::Thursday.readable(),
        &[Token::BorrowedStr("Thursday")],
    );
    assert_tokens(&Weekday::Friday.readable(), &[Token::BorrowedStr("Friday")]);
    assert_tokens(
        &Weekday::Saturday.readable(),
        &[Token::BorrowedStr("Saturday")],
    );
    assert_tokens(&Weekday::Sunday.readable(), &[Token::BorrowedStr("Sunday")]);
    assert_de_tokens_error::<Readable<Weekday>>(
        &[Token::BorrowedStr("NotADay")],
        r#"invalid value: string "NotADay", expected a day of the week"#,
    );
    assert_de_tokens_error::<Readable<Weekday>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<Weekday>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected u8",
    );
}

#[test]
fn month() {
    use Month::*;
    assert_tokens(&January.compact(), &[Token::U8(1)]);
    assert_tokens(&February.compact(), &[Token::U8(2)]);
    assert_tokens(&March.compact(), &[Token::U8(3)]);
    assert_tokens(&April.compact(), &[Token::U8(4)]);
    assert_tokens(&May.compact(), &[Token::U8(5)]);
    assert_tokens(&June.compact(), &[Token::U8(6)]);
    assert_tokens(&July.compact(), &[Token::U8(7)]);
    assert_tokens(&August.compact(), &[Token::U8(8)]);
    assert_tokens(&September.compact(), &[Token::U8(9)]);
    assert_tokens(&October.compact(), &[Token::U8(10)]);
    assert_tokens(&November.compact(), &[Token::U8(11)]);
    assert_tokens(&December.compact(), &[Token::U8(12)]);
    assert_de_tokens_error::<Compact<Month>>(
        &[Token::U8(0)],
        "invalid value: integer `0`, expected a value in the range 1..=12",
    );

    assert_tokens(&January.readable(), &[Token::BorrowedStr("January")]);
    assert_tokens(&February.readable(), &[Token::BorrowedStr("February")]);
    assert_tokens(&March.readable(), &[Token::BorrowedStr("March")]);
    assert_tokens(&April.readable(), &[Token::BorrowedStr("April")]);
    assert_tokens(&May.readable(), &[Token::BorrowedStr("May")]);
    assert_tokens(&June.readable(), &[Token::BorrowedStr("June")]);
    assert_tokens(&July.readable(), &[Token::BorrowedStr("July")]);
    assert_tokens(&August.readable(), &[Token::BorrowedStr("August")]);
    assert_tokens(&September.readable(), &[Token::BorrowedStr("September")]);
    assert_tokens(&October.readable(), &[Token::BorrowedStr("October")]);
    assert_tokens(&November.readable(), &[Token::BorrowedStr("November")]);
    assert_tokens(&December.readable(), &[Token::BorrowedStr("December")]);
    assert_de_tokens_error::<Readable<Month>>(
        &[Token::BorrowedStr("NotAMonth")],
        r#"invalid value: string "NotAMonth", expected a month of the year"#,
    );
    assert_de_tokens_error::<Readable<Month>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected a borrowed string",
    );
    assert_de_tokens_error::<Compact<Month>>(
        &[Token::Bool(false)],
        "invalid type: boolean `false`, expected u8",
    );
}
