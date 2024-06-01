use std::cmp::max;

use crate::DesktopEntry;

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

/// Try to guess the best [`DesktopEntry`] match for the app id.
pub fn try_match_app_id<'a, 'b, I>(
    app_id: I,
    entries: &'b [DesktopEntry<'a>],
    options: MatchAppIdOptions,
) -> Option<&'b DesktopEntry<'a>>
where
    I: AsRef<str>,
{
    let mut max_score = None;
    let mut second_max_score = 0.;

    for de in entries {
        let score = match_entry(app_id.as_ref(), de);

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

/// From 0 to 1.
/// 1 is a perfect match.
fn match_entry(id: &str, de: &DesktopEntry) -> f32 {
    let cmp = |id, de| {
        let lcsstr = textdistance::str::lcsstr(id, de);
        lcsstr as f32 / (max(id.len(), de.len())) as f32
    };

    fn max_f32(a: f32, b: f32) -> f32 {
        if a > b {
            a
        } else {
            b
        }
    }

    let id = id.to_lowercase();
    let de_id = de.appid.to_lowercase();
    let de_wm_class = de.startup_wm_class().unwrap_or_default().to_lowercase();
    let de_name = de.name(None).unwrap_or_default().to_lowercase();

    max_f32(
        cmp(&id, &de_id),
        max_f32(cmp(&id, &de_wm_class), cmp(&id, &de_name)),
    )
}
