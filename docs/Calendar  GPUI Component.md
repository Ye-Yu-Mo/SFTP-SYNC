---
title: "Calendar | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/calendar"
author:
published:
created: 2025-12-05
description: "A flexible calendar component for displaying months, navigating dates, and selecting single dates or date ranges."
tags:
  - "clippings"
---
## Calendar

A standalone calendar component that provides a rich interface for date selection and navigation. The Calendar component supports single date selection, date range selection, multiple month views, custom disabled dates, and comprehensive keyboard navigation.

- [CalendarState](https://docs.rs/gpui-component/latest/gpui_component/calendar/struct.CalendarState.html): For managing calendar state and selection.
- [Calendar](https://docs.rs/gpui-component/latest/gpui_component/calendar/struct.Calendar.html): For rendering the calendar UI.

## Import

```rust
rustuse gpui_component::{

    calendar::{Calendar, CalendarState, CalendarEvent, Date, Matcher},

};
```

## Usage

### Basic Calendar

```rust
rustlet state = cx.new(|cx| CalendarState::new(window, cx));

Calendar::new(&state)
```

### Calendar with Initial Date

```rust
rustuse chrono::Local;

let state = cx.new(|cx| {

    let mut state = CalendarState::new(window, cx);

    state.set_date(Local::now().naive_local().date(), window, cx);

    state

});

Calendar::new(&state)
```

### Date Range Calendar

```rust
rustuse chrono::{Local, Days};

let state = cx.new(|cx| {

    let mut state = CalendarState::new(window, cx);

    let now = Local::now().naive_local().date();

    state.set_date(

        Date::Range(Some(now), now.checked_add_days(Days::new(7))),

        window,

        cx

    );

    state

});

Calendar::new(&state)
```

### Multiple Months Display

```rust
rust// Show 2 months side by side

Calendar::new(&state)

    .number_of_months(2)

// Show 3 months

Calendar::new(&state)

    .number_of_months(3)
```

### Calendar Sizes

```rust
rustCalendar::new(&state).large()

Calendar::new(&state) // medium (default)

Calendar::new(&state).small()
```

## Date Restrictions

### Disabled Weekends

```rust
rustlet state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(vec![0, 6]) // Sunday=0, Saturday=6

});

Calendar::new(&state)
```

### Disabled Specific Weekdays

```rust
rust// Disable Sundays, Wednesdays, and Saturdays

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(vec![0, 3, 6])

});

Calendar::new(&state)
```

### Disabled Date Range

```rust
rustuse chrono::{Local, Days};

let now = Local::now().naive_local().date();

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::range(

            Some(now),

            now.checked_add_days(Days::new(7)),

        ))

});

Calendar::new(&state)
```

### Disabled Date Interval

```rust
rust// Disable dates outside the interval (before/after specified dates)

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::interval(

            Some(now.checked_sub_days(Days::new(30)).unwrap()),

            now.checked_add_days(Days::new(30))

        ))

});

Calendar::new(&state)
```

### Custom Disabled Dates

```rust
rust// Disable first 5 days of each month

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::custom(|date| {

            date.day0() < 5 // day0() returns 0-based day

        }))

});

Calendar::new(&state)

// Disable all Mondays

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::custom(|date| {

            date.weekday() == chrono::Weekday::Mon

        }))

});

Calendar::new(&state)

// Disable past dates

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::custom(|date| {

            *date < Local::now().naive_local().date()

        }))

});

Calendar::new(&state)
```

## Month/Year Navigation

The Calendar automatically provides navigation controls:

- **Previous/Next Month**: Arrow buttons in the header
- **Month Selection**: Click on month name to open month picker
- **Year Selection**: Click on year to open year picker
- **Year Pages**: Navigate through 20-year pages in year view

### Custom Year Range

```rust
rustlet state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .year_range((2020, 2030)) // Limit to specific year range

});

Calendar::new(&state)
```

## Handle Selection Events

## Advanced Examples

### Business Days Only Calendar

```rust
rustuse chrono::Weekday;

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::custom(|date| {

            matches!(date.weekday(), Weekday::Sat | Weekday::Sun)

        }))

});

Calendar::new(&state)
```

### Holiday Calendar

```rust
rustuse chrono::NaiveDate;

use std::collections::HashSet;

// Define holidays

let holidays: HashSet<NaiveDate> = [

    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // New Year

    NaiveDate::from_ymd_opt(2024, 7, 4).unwrap(), // Independence Day

    NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(), // Christmas

].into_iter().collect();

let state = cx.new(|cx| {

    CalendarState::new(window, cx)

        .disabled_matcher(Matcher::custom(move |date| {

            holidays.contains(date)

        }))

});

Calendar::new(&state)
```

### Multi-Month Range Selector

```rust
rustlet state = cx.new(|cx| {

    let mut state = CalendarState::new(window, cx);

    state.set_date(Date::Range(None, None), window, cx); // Range mode

    state

});

Calendar::new(&state)

    .number_of_months(3) // Show 3 months for easier range selection
```

### Quarterly View Calendar

```rust
rustlet state = cx.new(|cx| CalendarState::new(window, cx));

// Update to show current quarter's months

Calendar::new(&state)

    .number_of_months(3)
```

## Custom Styling

```rust
rustuse gpui::{px, relative};

Calendar::new(&calendar)

    .p_4() // Custom padding

    .bg(cx.theme().secondary) // Custom background

    .border_2() // Custom border

    .border_color(cx.theme().primary) // Custom border color

    .rounded(px(12.)) // Custom border radius

    .w(px(400.)) // Custom width

    .h(px(350.)) // Custom height
```

## API Reference

- [Calendar](https://docs.rs/gpui-component/latest/gpui_component/calendar/struct.Calendar.html)
- [CalendarState](https://docs.rs/gpui-component/latest/gpui_component/calendar/struct.CalendarState.html)
- [RangeMatcher](https://docs.rs/gpui-component/latest/gpui_component/calendar/struct.RangeMatcher.html)

## Examples

### Event Planning Calendar

```rust
rustlet event_calendar = cx.new(|cx| {

    let mut state = CalendarState::new(window, cx);

    // Disable past dates and weekends

    state = state.disabled_matcher(Matcher::custom(|date| {

        let now = Local::now().naive_local().date();

        *date < now || matches!(date.weekday(), Weekday::Sat | Weekday::Sun)

    }));

    state

});

Calendar::new(&event_calendar)

    .large() // Easier to see and interact with
```

### Vacation Booking Calendar

```rust
rustlet vacation_calendar = cx.new(|cx| {

    let mut state = CalendarState::new(window, cx);

    state.set_date(Date::Range(None, None), window, cx); // Range mode

    state

});

Calendar::new(&vacation_calendar)

    .number_of_months(2) // Show 2 months for range selection
```

### Report Date Range Selector

```rust
rustlet report_calendar = cx.new(|cx| {

    let mut state = CalendarState::new(window, cx)

        .year_range((2020, 2025)); // Limit to business years

    state.set_date(Date::Range(None, None), window, cx);

    state

});

Calendar::new(&report_calendar)

    .number_of_months(3)

    .small() // Compact for dashboard use
```

### Availability Calendar

The Calendar component provides a foundation for any date-related UI requirements, from simple date pickers to complex scheduling interfaces.