use std::cmp::max;

use crate::DesktopEntry;

#[inline]
fn add_value(v: &mut Vec<String>, value: &str, is_multiple: bool) {
    if is_multiple {
        value.split(';').for_each(|e| v.push(e.to_lowercase()));
    } else {
        v.push(value.to_lowercase());
    }
}

/// The returned value is between 0.0 and 1.0 (higher value means more similar).
/// You can use the `additional_values` parameter to add runtime string.
pub fn get_entry_score<'a, Q, L>(
    query: Q,
    entry: &'a DesktopEntry<'a>,
    locales: &[L],
    additional_values: &'a [&'a str],
) -> f64
where
    Q: AsRef<str>,
    L: AsRef<str>,
{
    // (field name, is separated by ";")
    let fields = [
        ("Name", false),
        ("GenericName", false),
        ("Comment", false),
        ("Categories", true),
        ("Keywords", true),
    ];

    let mut normalized_values: Vec<String> = Vec::new();

    normalized_values.extend(additional_values.iter().map(|val| val.to_lowercase()));

    let desktop_entry_group = entry.groups.get("Desktop Entry");

    for field in fields {
        if let Some(group) = desktop_entry_group {
            if let Some((default_value, locale_map)) = group.get(field.0) {
                add_value(&mut normalized_values, default_value, field.1);

                let mut at_least_one_locale = false;

                for locale in locales {
                    match locale_map.get(locale.as_ref()) {
                        Some(value) => {
                            add_value(&mut normalized_values, value, field.1);
                            at_least_one_locale = true;
                        }
                        None => {
                            if let Some(pos) = locale.as_ref().find('_') {
                                if let Some(value) = locale_map.get(&locale.as_ref()[..pos]) {
                                    add_value(&mut normalized_values, value, field.1);
                                    at_least_one_locale = true;
                                }
                            }
                        }
                    }
                }

                if !at_least_one_locale {
                    if let Some(domain) = &entry.ubuntu_gettext_domain {
                        let gettext_value = crate::dgettext(domain, default_value);
                        if !gettext_value.is_empty() {
                            add_value(&mut normalized_values, &gettext_value, false);
                        }
                    }
                }
            }
        }
    }

    let query = query.as_ref().to_lowercase();

    normalized_values
        .into_iter()
        .map(|de| strsim::jaro_winkler(&query, &de))
        .max_by(|e1, e2| e1.total_cmp(e2))
        .unwrap_or(0.0)
}

fn compare_str<'a>(pattern: &'a str, de_value: &'a str) -> f64 {
    let lcsstr = textdistance::str::lcsstr(pattern, de_value);

    lcsstr as f64 / (max(pattern.len(), de_value.len())) as f64
}

/// From 0 to 1.
/// 1 is a perfect match.
fn match_entry_from_id(pattern: &str, de: &DesktopEntry) -> f64 {
    let de_id = de.appid.to_lowercase();
    let de_wm_class = de.startup_wm_class().unwrap_or_default().to_lowercase();

    *[de_id, de_wm_class]
        .map(|de| compare_str(pattern, &de))
        .iter()
        .max_by(|e1, e2| e1.total_cmp(e2))
        .unwrap_or(&0.0)
}

#[derive(Debug, Clone)]
pub struct MatchAppIdOptions {
    /// Minimal score required to validate a match.
    /// Must be between 0 and 1
    pub min_score: f64,
    /// Optional field to lower the minimal score required to match
    /// if the entropy is at a certain value, e.i if the two best matches
    /// hare very different.
    /// - First element - minimal entropy, between 0 and 1
    /// - Second element - minimal score, between 0 and 1
    pub entropy: Option<(f64, f64)>,
}

impl Default for MatchAppIdOptions {
    fn default() -> Self {
        Self {
            min_score: 0.7,
            entropy: Some((0.15, 0.2)),
        }
    }
}

/// Return the best match over all provided [`DesktopEntry`].
/// Use this to match over the values provided by the compositor, not the user.
/// First entries get the priority.
pub fn get_best_match<'a, I>(
    patterns: &[I],
    entries: &'a [DesktopEntry<'a>],
    options: MatchAppIdOptions,
) -> Option<&'a DesktopEntry<'a>>
where
    I: AsRef<str>,
{
    let mut max_score = None;
    let mut second_max_score = 0.;

    let normalized_patterns = patterns
        .iter()
        .map(|e| e.as_ref().to_lowercase())
        .collect::<Vec<_>>();

    for de in entries {
        let score = normalized_patterns
            .iter()
            .map(|p| match_entry_from_id(p, de))
            .max_by(|e1, e2| e1.total_cmp(e2))
            .unwrap_or(0.0);

        match max_score {
            Some((prev_max_score, _)) => {
                if prev_max_score < score {
                    second_max_score = prev_max_score;
                    max_score = Some((score, de));
                }
            }
            None => {
                max_score = Some((score, de));
            }
        }

        if score > 0.99 {
            break;
        }
    }

    if let Some((max_score, de)) = max_score {
        if max_score > options.min_score {
            Some(de)
        } else if let Some((min_entropy, min_score_entropy)) = options.entropy {
            let entropy = max_score - second_max_score;

            if entropy > min_entropy && max_score > min_score_entropy {
                Some(de)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
