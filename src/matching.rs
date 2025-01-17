// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::DesktopEntry;

impl DesktopEntry {
    /// The returned value is between 0.0 and 1.0 (higher value means more similar).
    /// You can use the `additional_haystack_values` parameter to add relevant string that are not part of the desktop entry.
    pub fn match_query<Q, L>(
        &self,
        query: Q,
        locales: &[L],
        additional_haystack_values: &[&str],
    ) -> f64
    where
        Q: AsRef<str>,
        L: AsRef<str>,
    {
        #[inline]
        fn add_value(v: &mut Vec<String>, value: &str, is_multiple: bool) {
            if is_multiple {
                value.split(';').for_each(|e| v.push(e.to_lowercase()));
            } else {
                v.push(value.to_lowercase());
            }
        }

        // (field name, is separated by ";")
        let fields = [
            ("Name", false),
            ("GenericName", false),
            ("Comment", false),
            ("Categories", true),
            ("Keywords", true),
        ];

        let mut normalized_values: Vec<String> = Vec::new();

        normalized_values.extend(
            additional_haystack_values
                .iter()
                .map(|val| val.to_lowercase()),
        );

        let desktop_entry_group = self.groups.group("Desktop Entry");

        for field in fields {
            if let Some(group) = desktop_entry_group {
                if let Some((default_value, locale_map)) = group.0.get(field.0) {
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
                        if let Some(domain) = &self.ubuntu_gettext_domain {
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

        let query_espaced = query.split_ascii_whitespace().collect::<Vec<_>>();

        normalized_values
            .into_iter()
            .map(|de_field| {
                let jaro_score = strsim::jaro_winkler(&query, &de_field);

                if query_espaced.iter().any(|query| de_field.contains(*query)) {
                    // provide a bonus if the query is contained in the de field
                    (jaro_score + 0.1).clamp(0.61, 1.)
                } else {
                    jaro_score
                }
            })
            .max_by(|e1, e2| e1.total_cmp(e2))
            .unwrap_or(0.0)
    }
}

/// Return the corresponding [`DesktopEntry`] that match the given appid.
pub fn find_entry_from_appid<'a, I>(entries: I, appid: &str) -> Option<&'a DesktopEntry>
where
    I: IntoIterator<Item = &'a DesktopEntry>,
{
    let normalized_appid = appid.to_lowercase();

    entries.into_iter().find(|e| {
        if e.appid.to_lowercase() == normalized_appid {
            return true;
        }

        if let Some(field) = e.startup_wm_class() {
            if field.to_lowercase() == normalized_appid {
                return true;
            }
        }

        false
    })
}
