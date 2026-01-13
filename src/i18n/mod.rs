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
        t_with_vars(locale, "time.weeksAgo", &[("weeks", &weeks.to_string())])
    } else if days < 365 {
        let months = days / 30;
        t_with_vars(locale, "time.monthsAgo", &[("months", &months.to_string())])
    } else {
        let years = days / 365;
        t_with_vars(locale, "time.yearsAgo", &[("years", &years.to_string())])
    }
}
