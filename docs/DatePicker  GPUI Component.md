---
title: "DatePicker | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/date-picker"
author:
published:
created: 2025-12-05
description: "A date picker component for selecting single dates or date ranges with calendar interface."
tags:
  - "clippings"
---
## DatePicker

A flexible date picker component with calendar interface that supports single date selection, date range selection, custom date formatting, disabled dates, and preset ranges.

## Import

```rust
rustuse gpui_component::{

    date_picker::{DatePicker, DatePickerState, DateRangePreset, DatePickerEvent},

    calendar::{Date, Matcher},

};
```

## Usage

### Basic Date Picker

```rust
rustlet date_picker = cx.new(|cx| DatePickerState::new(window, cx));

DatePicker::new(&date_picker)
```

### With Initial Date

```rust
rustuse chrono::Local;

let date_picker = cx.new(|cx| {

    let mut picker = DatePickerState::new(window, cx);

    picker.set_date(Local::now().naive_local().date(), window, cx);

    picker

});

DatePicker::new(&date_picker)
```

### Date Range Picker

```rust
rustuse chrono::{Local, Days};

// Range mode picker

let range_picker = cx.new(|cx| DatePickerState::range(window, cx));

DatePicker::new(&range_picker)

    .number_of_months(2) // Show 2 months for easier range selection

// With initial range

let range_picker = cx.new(|cx| {

    let now = Local::now().naive_local().date();

    let mut picker = DatePickerState::new(window, cx);

    picker.set_date(

        (now, now.checked_add_days(Days::new(7)).unwrap()),

        window,

        cx,

    );

    picker

});

DatePicker::new(&range_picker)

    .number_of_months(2)
```

### With Custom Date Format

```rust
rustlet date_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .date_format("%Y-%m-%d") // ISO format

});

DatePicker::new(&date_picker)

// Other format examples:

// "%m/%d/%Y" -> 12/25/2023

// "%B %d, %Y" -> December 25, 2023

// "%d %b %Y" -> 25 Dec 2023
```

### With Placeholder

```rust
rustDatePicker::new(&date_picker)

    .placeholder("Select a date...")
```

### Cleanable Date Picker

```rust
rustDatePicker::new(&date_picker)

    .cleanable(true) // Show clear button when date is selected
```

### Different Sizes

```rust
rustDatePicker::new(&date_picker).large()

DatePicker::new(&date_picker) // medium (default)

DatePicker::new(&date_picker).small()
```

### Disabled State

```rust
rustDatePicker::new(&date_picker).disabled(true)
```

### Custom Appearance

```rust
rust// Without default styling

DatePicker::new(&date_picker).appearance(false)

// Use in custom container

div()

    .border_b_2()

    .px_6()

    .py_3()

    .border_color(cx.theme().border)

    .bg(cx.theme().secondary)

    .child(DatePicker::new(&date_picker).appearance(false))
```

## Date Restrictions

### Disabled Weekends

```rust
rustuse gpui_component::calendar;

let date_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .disabled_matcher(vec![0, 6]) // Sunday=0, Saturday=6

});

DatePicker::new(&date_picker)
```

### Disabled Date Range

```rust
rustuse chrono::{Local, Days};

let now = Local::now().naive_local().date();

let date_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .disabled_matcher(calendar::Matcher::range(

            Some(now),

            now.checked_add_days(Days::new(7)),

        ))

});

DatePicker::new(&date_picker)
```

### Disabled Date Interval

```rust
rustlet date_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .disabled_matcher(calendar::Matcher::interval(

            Some(now),

            now.checked_add_days(Days::new(5))

        ))

});

DatePicker::new(&date_picker)
```

### Custom Disabled Dates

```rust
rust// Disable first 5 days of each month

let date_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .disabled_matcher(calendar::Matcher::custom(|date| {

            date.day0() < 5

        }))

});

DatePicker::new(&date_picker)

// Disable all Mondays

let date_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .disabled_matcher(calendar::Matcher::custom(|date| {

            date.weekday() == chrono::Weekday::Mon

        }))

});
```

## Preset Ranges

### Single Date Presets

```rust
rustuse chrono::{Utc, Duration};

let presets = vec![

    DateRangePreset::single(

        "Yesterday",

        (Utc::now() - Duration::days(1)).naive_local().date(),

    ),

    DateRangePreset::single(

        "Last Week",

        (Utc::now() - Duration::weeks(1)).naive_local().date(),

    ),

    DateRangePreset::single(

        "Last Month",

        (Utc::now() - Duration::days(30)).naive_local().date(),

    ),

];

DatePicker::new(&date_picker)

    .presets(presets)
```

### Date Range Presets

```rust
rustlet range_presets = vec![

    DateRangePreset::range(

        "Last 7 Days",

        (Utc::now() - Duration::days(7)).naive_local().date(),

        Utc::now().naive_local().date(),

    ),

    DateRangePreset::range(

        "Last 30 Days",

        (Utc::now() - Duration::days(30)).naive_local().date(),

        Utc::now().naive_local().date(),

    ),

    DateRangePreset::range(

        "Last 90 Days",

        (Utc::now() - Duration::days(90)).naive_local().date(),

        Utc::now().naive_local().date(),

    ),

];

DatePicker::new(&date_picker)

    .number_of_months(2)

    .presets(range_presets)
```

## Handle Date Selection Events

## Multiple Months Display

```rust
rust// Show 2 months side by side (useful for date ranges)

DatePicker::new(&date_picker)

    .number_of_months(2)

// Show 3 months

DatePicker::new(&date_picker)

    .number_of_months(3)
```

## Advanced Examples

### Business Days Only

```rust
rustuse chrono::Weekday;

let business_days_picker = cx.new(|cx| {

    DatePickerState::new(window, cx)

        .disabled_matcher(calendar::Matcher::custom(|date| {

            matches!(date.weekday(), Weekday::Sat | Weekday::Sun)

        }))

});

DatePicker::new(&business_days_picker)

    .placeholder("Select business day")
```

### Date Range with Max Duration

### Quarter Presets

```rust
rustuse chrono::{NaiveDate, Datelike};

fn quarter_start(year: i32, quarter: u32) -> NaiveDate {

    let month = (quarter - 1) * 3 + 1;

    NaiveDate::from_ymd_opt(year, month, 1).unwrap()

}

fn quarter_end(year: i32, quarter: u32) -> NaiveDate {

    let month = quarter * 3;

    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    NaiveDate::from_ymd_opt(year, month, start.days_in_month()).unwrap()

}

let year = Local::now().year();

let quarterly_presets = vec![

    DateRangePreset::range("Q1", quarter_start(year, 1), quarter_end(year, 1)),

    DateRangePreset::range("Q2", quarter_start(year, 2), quarter_end(year, 2)),

    DateRangePreset::range("Q3", quarter_start(year, 3), quarter_end(year, 3)),

    DateRangePreset::range("Q4", quarter_start(year, 4), quarter_end(year, 4)),

];

DatePicker::new(&date_picker)

    .presets(quarterly_presets)
```

## Examples

### Event Date Picker

```rust
rustlet event_date = cx.new(|cx| {

    let mut picker = DatePickerState::new(window, cx)

        .date_format("%B %d, %Y")

        .disabled_matcher(calendar::Matcher::custom(|date| {

            // Disable past dates

            *date < Local::now().naive_local().date()

        }));

    picker

});

DatePicker::new(&event_date)

    .placeholder("Choose event date")

    .cleanable(true)
```

### Booking System Date Range

```rust
rustlet booking_range = cx.new(|cx| DatePickerState::range(window, cx));

let booking_presets = vec![

    DateRangePreset::range("This Weekend", /* weekend dates */),

    DateRangePreset::range("Next Week", /* next week dates */),

    DateRangePreset::range("This Month", /* this month dates */),

];

DatePicker::new(&booking_range)

    .number_of_months(2)

    .presets(booking_presets)

    .placeholder("Select check-in and check-out dates")
```

### Financial Period Selector

```rust
rustlet financial_period = cx.new(|cx| {

    DatePickerState::range(window, cx)

        .date_format("%Y-%m-%d")

});

DatePicker::new(&financial_period)

    .number_of_months(3)

    .presets(quarterly_presets)

    .placeholder("Select reporting period")
```