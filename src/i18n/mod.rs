#![allow(dead_code)]

mod translations;

use std::collections::HashMap;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Locale {
    #[default]
    En,
    Ja,
    ZhHans,
    ZhHant,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ja => "ja",
            Locale::ZhHans => "zh-Hans",
            Locale::ZhHant => "zh-Hant",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Ja => "日本語",
            Locale::ZhHans => "简体中文",
            Locale::ZhHant => "繁體中文",
        }
    }

    pub fn all() -> &'static [Locale] {
        &[Locale::En, Locale::Ja, Locale::ZhHans, Locale::ZhHant]
    }
}

/// Translation function - gets a translation for the given key
pub fn t(locale: Locale, key: &str) -> String {
    get_translation(locale, key)
}

/// Translation function with variable substitution
pub fn t_with_vars(locale: Locale, key: &str, vars: &[(&str, &str)]) -> String {
    let mut result = get_translation(locale, key);
    for (name, value) in vars {
        result = result.replace(&format!("{{{}}}", name), value);
    }
    result
}

/// Get translation for a key, falling back to English if not found
fn get_translation(locale: Locale, key: &str) -> String {
    let translations = get_translations(locale);
    translations
        .get(key)
        .copied()
        .or_else(|| {
            // Fallback to English
            if locale != Locale::En {
                get_translations(Locale::En).get(key).copied()
            } else {
                None
            }
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| key.to_string())
}

fn get_translations(locale: Locale) -> &'static HashMap<&'static str, &'static str> {
    match locale {
        Locale::En => translations::EN,
        Locale::Ja => translations::JA,
        Locale::ZhHans => translations::ZH_HANS,
        Locale::ZhHant => translations::ZH_HANT,
    }
}

/// Format relative time
pub fn format_relative_time(locale: Locale, days: i64) -> String {
    if days == 0 {
        t(locale, "time.today")
    } else if days == 1 {
        t(locale, "time.yesterday")
    } else if days < 7 {
        t_with_vars(locale, "time.daysAgo", &[("days", &days.to_string())])
    } else if days < 30 {
        let weeks = days / 7;
        if weeks == 1 {
            t(locale, "time.weekAgo")
        } else {
            t_with_vars(locale, "time.weeksAgo", &[("weeks", &weeks.to_string())])
        }
    } else if days < 365 {
        let months = days / 30;
        if months == 1 {
            t(locale, "time.monthAgo")
        } else {
            t_with_vars(locale, "time.monthsAgo", &[("months", &months.to_string())])
        }
    } else {
        let years = days / 365;
        if years == 1 {
            t(locale, "time.yearAgo")
        } else {
            t_with_vars(locale, "time.yearsAgo", &[("years", &years.to_string())])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_code() {
        assert_eq!(Locale::En.code(), "en");
        assert_eq!(Locale::Ja.code(), "ja");
        assert_eq!(Locale::ZhHans.code(), "zh-Hans");
        assert_eq!(Locale::ZhHant.code(), "zh-Hant");
    }

    #[test]
    fn test_locale_display_name() {
        assert_eq!(Locale::En.display_name(), "English");
        assert_eq!(Locale::Ja.display_name(), "日本語");
        assert_eq!(Locale::ZhHans.display_name(), "简体中文");
        assert_eq!(Locale::ZhHant.display_name(), "繁體中文");
    }

    #[test]
    fn test_locale_all() {
        let all = Locale::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Locale::En));
        assert!(all.contains(&Locale::Ja));
        assert!(all.contains(&Locale::ZhHans));
        assert!(all.contains(&Locale::ZhHant));
    }

    #[test]
    fn test_locale_default() {
        let default: Locale = Default::default();
        assert_eq!(default, Locale::En);
    }

    #[test]
    fn test_locale_equality() {
        assert_eq!(Locale::En, Locale::En);
        assert_ne!(Locale::En, Locale::Ja);
    }

    #[test]
    fn test_translation_basic() {
        // Test that we get a translation (not the key back)
        let result = t(Locale::En, "app.name");
        // The translation should exist
        assert!(!result.is_empty());
    }

    #[test]
    fn test_translation_fallback_to_english() {
        // Test fallback: use a key that exists in English
        let en_result = t(Locale::En, "app.name");
        let ja_result = t(Locale::Ja, "app.name");

        // Both should return something (not empty)
        assert!(!en_result.is_empty());
        assert!(!ja_result.is_empty());
    }

    #[test]
    fn test_translation_missing_key_returns_key() {
        let result = t(Locale::En, "nonexistent.key.that.does.not.exist");
        assert_eq!(result, "nonexistent.key.that.does.not.exist");
    }

    #[test]
    fn test_t_with_vars_substitution() {
        // Test variable substitution
        let result = t_with_vars(Locale::En, "time.daysAgo", &[("days", "5")]);
        assert!(result.contains("5"));
    }

    #[test]
    fn test_format_relative_time_today() {
        let result = format_relative_time(Locale::En, 0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_relative_time_yesterday() {
        let result = format_relative_time(Locale::En, 1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_relative_time_days() {
        let result = format_relative_time(Locale::En, 3);
        assert!(result.contains("3"));
    }

    #[test]
    fn test_format_relative_time_weeks() {
        let result = format_relative_time(Locale::En, 14);
        assert_eq!(result, "2 weeks ago");
    }

    #[test]
    fn test_format_relative_time_months() {
        let result = format_relative_time(Locale::En, 60);
        assert_eq!(result, "2 months ago");
    }

    #[test]
    fn test_format_relative_time_years() {
        let result = format_relative_time(Locale::En, 400);
        assert_eq!(result, "1 year ago");
    }

    #[test]
    fn test_format_relative_time_singular_week() {
        let result = format_relative_time(Locale::En, 7);
        assert_eq!(result, "1 week ago");
    }

    #[test]
    fn test_format_relative_time_singular_month() {
        let result = format_relative_time(Locale::En, 30);
        assert_eq!(result, "1 month ago");
    }
}
