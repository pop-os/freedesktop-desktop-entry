use std::cmp::max;

use crate::DesktopEntry;

/// From 0 to 1.
/// 1 is a perfect match.
fn match_entry_(query: &str, de: &DesktopEntry, _languages: &[&str]) -> f32 {
    let cmp = |query, de| {
        let lcsstr = textdistance::str::lcsstr(query, de);
        lcsstr as f32 / (max(query.len(), de.len())) as f32
    };

    fn max_f32(a: f32, b: f32) -> f32 {
        if a > b {
            a
        } else {
            b
        }
    }

    // should search in
    // - id
    // - name
    // - coment
    // - generic name
    // - keyword
    // - categories
    // - wm_class
    let de_id = de.appid.to_lowercase();
    let de_wm_class = de.startup_wm_class().unwrap_or_default().to_lowercase();
    let de_name = de.name(None).unwrap_or_default().to_lowercase();

    max_f32(
        cmp(query, &de_id),
        max_f32(cmp(query, &de_wm_class), cmp(query, &de_name)),
    )
}

pub struct MatchAppIdOptions {
    /// Minimal score required to validate a match.
    /// Must be between 0 and 1
    pub min_score: f32,
    /// Optional field to lower the minimal score required to match
    /// if the entropy is at a certain value, e.i if the two best matches
    /// hare very different.
    /// - First element - minimal entropy, between 0 and 1
    /// - Second element - minimal score, between 0 and 1
    pub entropy: Option<(f32, f32)>,
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
/// Use this to match over the values provided by the compositor,
/// not the user.
pub fn get_best_match<'a, 'l, I>(
    patterns: &'a [I],
    entries: &'a [DesktopEntry<'a>],
    options: MatchAppIdOptions,
) -> Option<&'a DesktopEntry<'a>>
where
    I: AsRef<str>,
{
    todo!()
}

/// Return a score between 0 and 1
pub fn get_entry_score<'a, 'l, I>(
    query: I,
    entry: &'a DesktopEntry<'a>,
    languages: &'l [&'l str],
) -> f32
where
    I: AsRef<str>,
{
    todo!()
}



// todo: use rayon
/// Try to guess the best [`DesktopEntry`] match for a query.
fn try_match_entries<'a, 'l, I>(
    query: I,
    entries: &'a [DesktopEntry<'a>],
    options: MatchAppIdOptions,
    languages: &'l [&'l str],
) -> Option<&'a DesktopEntry<'a>>
where
    I: AsRef<str>,
{
    let mut max_score = None;
    let mut second_max_score = 0.;

    let normalized_query = query.as_ref().to_lowercase();

    for de in entries {
        let score = match_entry_(&normalized_query, de, languages);

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
