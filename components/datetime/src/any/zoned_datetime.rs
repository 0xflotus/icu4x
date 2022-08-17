// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::{calendar, options::DateTimeFormatterOptions, raw};
use alloc::string::String;

use icu_provider::prelude::*;

use crate::input::{DateTimeInput, ExtractedDateTimeInput, TimeZoneInput};
use crate::provider::{self, calendar::*, date_time::PatternSelector, week_data::WeekDataV1Marker};
use crate::time_zone::TimeZoneFormatterOptions;
use crate::{DateTimeFormatterError, FormattedZonedDateTime};
use icu_calendar::any_calendar::{AnyCalendar, AnyCalendarKind};
use icu_calendar::provider::{JapaneseErasV1Marker, JapaneseExtendedErasV1Marker};
use icu_calendar::{types::Time, DateTime};
use icu_decimal::provider::DecimalSymbolsV1Marker;
use icu_plurals::provider::OrdinalV1Marker;

/// [`ZonedDateTimeFormatter`] is a formatter capable of formatting
/// date/times with timezones from any calendar, selected at runtime. For the difference between this and [`TypedZonedDateTimeFormatter`](crate::TypedZonedDateTimeFormatter),
/// please read the [crate root docs][crate].
///
/// This is equivalently the composition of
/// [`DateTimeFormatter`](crate::DateTimeFormatter) and [`TimeZoneFormatter`](crate::TimeZoneFormatter).
///
/// [`ZonedDateTimeFormatter`] uses data from the [data provider]s, the selected [`DataLocale`], and the
/// provided pattern to collect all data necessary to format a datetime with time zones into that locale.
///
/// The various pattern symbols specified in UTS-35 require different sets of data for formatting.
/// As such, `TimeZoneFormatter` will pull in only the resources it needs to format that pattern
/// that is derived from the provided [`DateTimeFormatterOptions`].
///
/// For that reason, one should think of the process of formatting a zoned datetime in two steps:
/// first, a computationally heavy construction of [`ZonedDateTimeFormatter`], and then fast formatting
/// of the data using the instance.
///
/// # Examples
///
/// ```
/// use icu::calendar::{DateTime, Gregorian};
/// use icu::timezone::CustomTimeZone;
/// use icu::datetime::{options::length, ZonedDateTimeFormatter};
/// use icu::locid::locale;
/// use icu_datetime::TimeZoneFormatterOptions;
///
/// let provider = icu_testdata::get_provider();
///
/// let options = length::Bag::from_date_time_style(length::Date::Medium, length::Time::Long);
/// let zdtf = ZonedDateTimeFormatter::try_new_with_buffer_provider(
///     &provider,
///     &locale!("en").into(),
///     options.into(),
///     TimeZoneFormatterOptions::default(),
/// )
/// .expect("Failed to create TypedDateTimeFormatter instance.");
///
/// let datetime = DateTime::new_gregorian_datetime(2020, 9, 1, 12, 34, 28)
///     .expect("Failed to construct DateTime.");
/// let any_datetime = datetime.to_any();
///
/// let time_zone: CustomTimeZone = "+05:00".parse().expect("Timezone should parse");
///
/// let value = zdtf.format_to_string(&any_datetime, &time_zone).expect("calendars should match");
///
/// assert_eq!(value, "Sep 1, 2020, 12:34:28 PM GMT+05:00");
/// ```
pub struct ZonedDateTimeFormatter(raw::ZonedDateTimeFormatter, AnyCalendar);

impl ZonedDateTimeFormatter {
    /// Constructor that takes a selected [`DataLocale`], a reference to a [data provider] for
    /// dates, a [data provider] for time zones, a [data provider] for calendars, and a list of [`DateTimeFormatterOptions`].
    /// It collects all data necessary to format zoned datetime values into the given locale.
    ///
    /// This method is **unstable**, more bounds may be added in the future as calendar support is added. It is
    /// preferable to use a provider that implements `DataProvider<D>` for all `D`, and ensure data is loaded as
    /// appropriate. The [`Self::try_new_with_buffer_provider()`], [`Self::try_new_with_any_provider()`] constructors
    /// may also be used if compile stability is desired.
    ///
    /// This method will pick the calendar off of the locale; and if unspecified or unknown will fall back to the default
    /// calendar for the locale. See [`AnyCalendarKind`] for a list of supported calendars.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::calendar::Gregorian;
    /// use icu::datetime::options::length;
    /// use icu::datetime::mock::parse_zoned_gregorian_from_str;
    /// use icu::datetime::{DateTimeFormatterOptions, ZonedDateTimeFormatter};
    /// use icu::locid::Locale;
    /// use icu::datetime::TimeZoneFormatterOptions;
    /// use std::str::FromStr;
    ///
    /// let provider = icu_testdata::get_provider();
    ///
    /// let options = length::Bag::from_date_time_style(length::Date::Medium, length::Time::Long).into();
    /// let locale = Locale::from_str("en-u-ca-gregory").unwrap();
    ///
    /// let zdtf = ZonedDateTimeFormatter::try_new_unstable(
    ///     &provider,
    ///     &locale.into(),
    ///     options,
    ///     TimeZoneFormatterOptions::default(),
    /// ).expect("Construction should succeed");
    ///
    /// let (datetime, time_zone) = parse_zoned_gregorian_from_str("2021-04-08T16:12:37.000-07:00")
    ///     .expect("Failed to parse zoned datetime");
    /// let any_datetime = datetime.to_any();
    ///
    /// assert_eq!(zdtf.format_to_string(&any_datetime, &time_zone).unwrap(), "Apr 8, 2021, 4:12:37 PM GMT-07:00");
    /// ```
    ///
    /// [data provider]: icu_provider
    #[cfg(feature = "experimental")]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn try_new_unstable<P>(
        provider: &P,
        locale: &DataLocale,
        date_time_format_options: DateTimeFormatterOptions,
        time_zone_format_options: TimeZoneFormatterOptions,
    ) -> Result<Self, DateTimeFormatterError>
    where
        P: DataProvider<TimeSymbolsV1Marker>
            + DataProvider<TimeLengthsV1Marker>
            + DataProvider<crate::provider::calendar::DateSkeletonPatternsV1Marker>
            + DataProvider<WeekDataV1Marker>
            + DataProvider<provider::time_zones::TimeZoneFormatsV1Marker>
            + DataProvider<provider::time_zones::ExemplarCitiesV1Marker>
            + DataProvider<provider::time_zones::MetaZoneGenericNamesLongV1Marker>
            + DataProvider<provider::time_zones::MetaZoneGenericNamesShortV1Marker>
            + DataProvider<provider::time_zones::MetaZoneSpecificNamesLongV1Marker>
            + DataProvider<provider::time_zones::MetaZoneSpecificNamesShortV1Marker>
            + DataProvider<OrdinalV1Marker>
            + DataProvider<DecimalSymbolsV1Marker>
            + DataProvider<GregorianDateLengthsV1Marker>
            + DataProvider<BuddhistDateLengthsV1Marker>
            + DataProvider<JapaneseDateLengthsV1Marker>
            + DataProvider<JapaneseExtendedDateLengthsV1Marker>
            + DataProvider<CopticDateLengthsV1Marker>
            + DataProvider<IndianDateLengthsV1Marker>
            + DataProvider<EthiopicDateLengthsV1Marker>
            + DataProvider<GregorianDateSymbolsV1Marker>
            + DataProvider<BuddhistDateSymbolsV1Marker>
            + DataProvider<JapaneseDateSymbolsV1Marker>
            + DataProvider<JapaneseExtendedDateSymbolsV1Marker>
            + DataProvider<CopticDateSymbolsV1Marker>
            + DataProvider<IndianDateSymbolsV1Marker>
            + DataProvider<EthiopicDateSymbolsV1Marker>
            + DataProvider<JapaneseErasV1Marker>
            + DataProvider<JapaneseExtendedErasV1Marker>
            + ?Sized,
    {
        // TODO(#2188): Avoid cloning the DataLocale by passing the calendar
        // separately into the raw formatter.
        let mut locale_with_cal = locale.clone();

        let calendar = AnyCalendar::try_new_for_locale_unstable(provider, &locale_with_cal)?;
        let kind = calendar.kind();
        kind.set_on_data_locale(&mut locale_with_cal);

        let patterns = PatternSelector::for_options(
            provider,
            calendar::load_lengths_for_any_calendar_kind(provider, locale, kind)?,
            &locale_with_cal,
            &date_time_format_options,
        )?;

        Ok(Self(
            raw::ZonedDateTimeFormatter::try_new(
                provider,
                patterns,
                || calendar::load_symbols_for_any_calendar_kind(provider, locale, kind),
                locale_with_cal,
                time_zone_format_options,
            )?,
            calendar,
        ))
    }

    #[allow(missing_docs)] // The docs use the "experimental" version
    #[cfg(not(feature = "experimental"))]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn try_new_unstable<P>(
        provider: &P,
        locale: &DataLocale,
        date_time_format_options: DateTimeFormatterOptions,
        time_zone_format_options: TimeZoneFormatterOptions,
    ) -> Result<Self, DateTimeFormatterError>
    where
        P: DataProvider<TimeSymbolsV1Marker>
            + DataProvider<TimeLengthsV1Marker>
            + DataProvider<WeekDataV1Marker>
            + DataProvider<provider::time_zones::TimeZoneFormatsV1Marker>
            + DataProvider<provider::time_zones::ExemplarCitiesV1Marker>
            + DataProvider<provider::time_zones::MetaZoneGenericNamesLongV1Marker>
            + DataProvider<provider::time_zones::MetaZoneGenericNamesShortV1Marker>
            + DataProvider<provider::time_zones::MetaZoneSpecificNamesLongV1Marker>
            + DataProvider<provider::time_zones::MetaZoneSpecificNamesShortV1Marker>
            + DataProvider<OrdinalV1Marker>
            + DataProvider<DecimalSymbolsV1Marker>
            + DataProvider<GregorianDateLengthsV1Marker>
            + DataProvider<BuddhistDateLengthsV1Marker>
            + DataProvider<JapaneseDateLengthsV1Marker>
            + DataProvider<JapaneseExtendedDateLengthsV1Marker>
            + DataProvider<CopticDateLengthsV1Marker>
            + DataProvider<IndianDateLengthsV1Marker>
            + DataProvider<EthiopicDateLengthsV1Marker>
            + DataProvider<GregorianDateSymbolsV1Marker>
            + DataProvider<BuddhistDateSymbolsV1Marker>
            + DataProvider<JapaneseDateSymbolsV1Marker>
            + DataProvider<JapaneseExtendedDateSymbolsV1Marker>
            + DataProvider<CopticDateSymbolsV1Marker>
            + DataProvider<IndianDateSymbolsV1Marker>
            + DataProvider<EthiopicDateSymbolsV1Marker>
            + DataProvider<JapaneseErasV1Marker>
            + DataProvider<JapaneseExtendedErasV1Marker>
            + ?Sized,
    {
        // TODO(#2188): Avoid cloning the DataLocale by passing the calendar
        // separately into the raw formatter.
        let mut locale_with_cal = locale.clone();

        let calendar = AnyCalendar::try_new_for_locale_unstable(provider, &locale_with_cal)?;
        let kind = calendar.kind();
        kind.set_on_data_locale(&mut locale_with_cal);

        let patterns = PatternSelector::for_options(
            provider,
            calendar::load_lengths_for_any_calendar_kind(provider, locale, kind)?,
            &locale_with_cal,
            &date_time_format_options,
        )?;

        Ok(Self(
            raw::ZonedDateTimeFormatter::try_new(
                provider,
                patterns,
                || calendar::load_symbols_for_any_calendar_kind(provider, locale, kind),
                locale_with_cal,
                time_zone_format_options,
            )?,
            calendar,
        ))
    }

    /// Construct a new [`ZonedDateTimeFormatter`] from a data provider that implements
    /// [`AnyProvider`].
    ///
    /// This method will pick the calendar off of the locale; and if unspecified or unknown will fall back to the default
    /// calendar for the locale. See [`AnyCalendarKind`] for a list of supported calendars.
    ///
    /// The provider must be able to provide data for the following keys: `datetime/symbols@1`, `datetime/timelengths@1`,
    /// `datetime/timelengths@1`, `datetime/symbols@1`, `datetime/skeletons@1`, `datetime/week_data@1`, `plurals/ordinals@1`,
    /// `time_zone/formats@1`, `time_zone/exemplar_cities@1`, `time_zone/generic_long@1`, `time_zone/generic_short@1`,
    /// `time_zone/specific_long@1`, `time_zone/specific_short@1`, `time_zone/metazone_period@1`.
    ///
    /// Furthermore, based on the type of calendar used, one of the following data keys may be necessary:
    ///
    /// - `u-ca-japanese` (Japanese calendar): `calendar/japanese@1`
    ///
    /// Test will currently fail due to <https://github.com/unicode-org/icu4x/issues/2188>,
    /// since these functions currently *must* be given a fallback-enabled provider and
    /// we do not have one in `icu_testdata`
    ///
    /// ```ignore
    /// use icu::calendar::Gregorian;
    /// use icu::datetime::options::length;
    /// use icu::datetime::mock::parse_zoned_gregorian_from_str;
    /// use icu::datetime::{DateTimeFormatterOptions, ZonedDateTimeFormatter};
    /// use icu::locid::Locale;
    /// use icu::datetime::TimeZoneFormatterOptions;
    /// use std::str::FromStr;
    ///
    /// let provider = icu_testdata::get_baked_provider();
    ///
    /// let options = length::Bag::from_date_time_style(length::Date::Medium, length::Time::Long).into();
    /// let locale = Locale::from_str("en-u-ca-gregory").unwrap();
    ///
    /// let zdtf = ZonedDateTimeFormatter::try_new_with_any_provider(
    ///     &provider,
    ///     &locale.into(),
    ///     options,
    ///     TimeZoneFormatterOptions::default(),
    /// ).expect("Construction should succeed");
    ///
    /// let (datetime, time_zone) = parse_zoned_gregorian_from_str("2021-04-08T16:12:37.000-07:00")
    ///     .expect("Failed to parse zoned datetime");
    /// let any_datetime = datetime.to_any();
    ///
    /// assert_eq!(zdtf.format_to_string(&any_datetime, &time_zone).unwrap(), "Apr 8, 2021, 4:12:37 PM GMT-07:00");
    /// ```
    #[inline]
    pub fn try_new_with_any_provider<P>(
        data_provider: &P,
        locale: &DataLocale,
        options: DateTimeFormatterOptions,
        time_zone_format_options: TimeZoneFormatterOptions,
    ) -> Result<Self, DateTimeFormatterError>
    where
        P: AnyProvider,
    {
        let downcasting = data_provider.as_downcasting();
        Self::try_new_unstable(&downcasting, locale, options, time_zone_format_options)
    }

    /// Construct a new [`ZonedDateTimeFormatter`] from a data provider that implements
    /// [`BufferProvider`].
    ///
    /// This method will pick the calendar off of the locale; and if unspecified or unknown will fall back to the default
    /// calendar for the locale. See [`AnyCalendarKind`] for a list of supported calendars.
    ///
    /// The provider must be able to provide data for the following keys: `datetime/symbols@1`, `datetime/timelengths@1`,
    /// `datetime/timelengths@1`, `datetime/symbols@1`, `datetime/skeletons@1`, `datetime/week_data@1`, `plurals/ordinals@1`,
    /// `time_zone/formats@1`, `time_zone/exemplar_cities@1`, `time_zone/generic_long@1`, `time_zone/generic_short@1`,
    /// `time_zone/specific_long@1`, `time_zone/specific_short@1`, `time_zone/metazone_period@1`.
    ///
    /// Furthermore, based on the type of calendar used, one of the following data keys may be necessary:
    ///
    /// - `u-ca-japanese` (Japanese calendar): `calendar/japanese@1`
    ///
    /// ```rust
    /// use icu::calendar::Gregorian;
    /// use icu::datetime::options::length;
    /// use icu::datetime::mock::parse_zoned_gregorian_from_str;
    /// use icu::datetime::{DateTimeFormatterOptions, ZonedDateTimeFormatter};
    /// use icu::locid::Locale;
    /// use icu::datetime::TimeZoneFormatterOptions;
    /// use std::str::FromStr;
    ///
    /// let provider = icu_testdata::get_provider();
    ///
    /// let options = length::Bag::from_date_time_style(length::Date::Medium, length::Time::Long).into();
    /// let locale = Locale::from_str("en").unwrap();
    ///
    /// let zdtf = ZonedDateTimeFormatter::try_new_with_buffer_provider(
    ///     &provider,
    ///     &locale.into(),
    ///     options,
    ///     TimeZoneFormatterOptions::default(),
    /// ).expect("Construction should succeed");
    ///
    /// let (datetime, time_zone) = parse_zoned_gregorian_from_str("2021-04-08T16:12:37.000-07:00")
    ///     .expect("Failed to parse zoned datetime");
    /// let any_datetime = datetime.to_any();
    ///
    /// assert_eq!(zdtf.format_to_string(&any_datetime, &time_zone).unwrap(), "Apr 8, 2021, 4:12:37 PM GMT-07:00");
    /// ```
    #[inline]
    #[cfg(feature = "serde")]
    pub fn try_new_with_buffer_provider<P>(
        data_provider: &P,
        locale: &DataLocale,
        options: DateTimeFormatterOptions,
        time_zone_format_options: TimeZoneFormatterOptions,
    ) -> Result<Self, DateTimeFormatterError>
    where
        P: BufferProvider,
    {
        let deserializing = data_provider.as_deserializing();
        Self::try_new_unstable(&deserializing, locale, options, time_zone_format_options)
    }

    /// Takes a [`DateTimeInput`] and a [`TimeZoneInput`] and returns an instance of a [`FormattedZonedDateTime`]
    /// that contains all information necessary to display a formatted date and operate on it.
    ///
    /// This function will fail if the date passed in uses a different calendar than that of the
    /// AnyCalendar. Please convert dates before passing them in if necessary. This function
    /// will automatically convert and format dates that are associated with the ISO calendar.
    #[inline]
    pub fn format<'l, T>(
        &'l self,
        date: &impl DateTimeInput<Calendar = AnyCalendar>,
        time_zone: &impl TimeZoneInput,
    ) -> Result<FormattedZonedDateTime<'l>, DateTimeFormatterError> {
        if let Some(converted) = self.convert_if_necessary(date)? {
            Ok(self.0.format(&converted, time_zone))
        } else {
            Ok(self.0.format(date, time_zone))
        }
    }

    /// Takes a mutable reference to anything that implements [`Write`](std::fmt::Write) trait
    /// and a [`DateTimeInput`] and a [`TimeZoneInput`] and populates the buffer with a formatted value.
    ///
    /// This function will fail if the date passed in uses a different calendar than that of the
    /// AnyCalendar. Please convert dates before passing them in if necessary. This function
    /// will automatically convert and format dates that are associated with the ISO calendar.
    #[inline]
    pub fn format_to_write(
        &self,
        w: &mut impl core::fmt::Write,
        date: &impl DateTimeInput<Calendar = AnyCalendar>,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatterError> {
        if let Some(converted) = self.convert_if_necessary(date)? {
            self.0.format_to_write(w, &converted, time_zone)?;
        } else {
            self.0.format_to_write(w, date, time_zone)?;
        }
        Ok(())
    }

    /// Takes a [`DateTimeInput`] and a [`TimeZoneInput`] and returns it formatted as a string.
    ///
    /// This function will fail if the date passed in uses a different calendar than that of the
    /// AnyCalendar. Please convert dates before passing them in if necessary. This function
    /// will automatically convert and format dates that are associated with the ISO calendar.
    #[inline]
    pub fn format_to_string(
        &self,
        date: &impl DateTimeInput<Calendar = AnyCalendar>,
        time_zone: &impl TimeZoneInput,
    ) -> Result<String, DateTimeFormatterError> {
        if let Some(converted) = self.convert_if_necessary(date)? {
            Ok(self.0.format_to_string(&converted, time_zone))
        } else {
            Ok(self.0.format_to_string(date, time_zone))
        }
    }

    /// Converts a date to the correct calendar if necessary
    ///
    /// Returns Err if the date is not ISO or compatible with the current calendar, returns Ok(None)
    /// if the date is compatible with the current calendar and doesn't need conversion
    fn convert_if_necessary(
        &self,
        value: &impl DateTimeInput<Calendar = AnyCalendar>,
    ) -> Result<Option<ExtractedDateTimeInput>, DateTimeFormatterError> {
        let this_calendar = self.1.kind();
        let date_calendar = value.any_calendar_kind();
        if Some(this_calendar) != date_calendar {
            if date_calendar != Some(AnyCalendarKind::Iso) {
                return Err(DateTimeFormatterError::MismatchedAnyCalendar(
                    this_calendar,
                    date_calendar,
                ));
            }
            let date = value.to_iso();
            let time = Time::new(
                value.hour().unwrap_or_default(),
                value.minute().unwrap_or_default(),
                value.second().unwrap_or_default(),
                value.nanosecond().unwrap_or_default(),
            );
            let datetime = DateTime::new(date, time).to_any();
            let converted = self.1.convert_any_datetime(&datetime);
            // FIXME(#2145) this is very hacky, can be improved after we improve ZonedDateTimeInput
            let converted = ExtractedDateTimeInput::extract_from(&converted);
            Ok(Some(converted))
        } else {
            Ok(None)
        }
    }
}