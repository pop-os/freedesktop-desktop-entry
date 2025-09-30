// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use crate::{DesktopEntry, Group};
use crate::{Groups, LocaleMap};
use bstr::ByteSlice;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("path does not contain a valid app ID")]
    AppID,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("MultipleGroupWithSameName")]
    MultipleGroupWithSameName,
    #[error("KeyValueWithoutAGroup")]
    KeyValueWithoutAGroup,
    #[error("InvalidKey. Accepted: A-Za-z0-9")]
    InvalidKey,
    #[error("KeyDoesNotExist, this can happens when a localized key has no default value")]
    KeyDoesNotExist,
    #[error("InvalidValue")]
    InvalidValue,
}

struct UnknownKey<'a> {
    key: &'a str,
    locale: String,
    value: String,
}

impl DesktopEntry {
    pub fn from_str<L>(
        path: impl Into<PathBuf>,
        input: &str,
        locales_filter: Option<&[L]>,
    ) -> Result<DesktopEntry, DecodeError>
    where
        L: AsRef<str>,
    {
        #[inline(never)]
        fn inner<'a>(
            path: PathBuf,
            input: &'a str,
            locales_filter: Option<Vec<&str>>,
        ) -> Result<DesktopEntry, DecodeError> {
            let path: PathBuf = path.into();

            let appid = get_app_id(&path)?;

            let mut groups = Groups::default();
            let mut active_group: Option<ActiveGroup> = None;
            let mut active_keys: Option<ActiveKeys> = None;
            let mut ubuntu_gettext_domain = None;

            let mut unknown_keys: Vec<UnknownKey> = Vec::new();

            for line in input.lines() {
                process_line(
                    line,
                    &mut groups,
                    &mut active_group,
                    &mut active_keys,
                    &mut ubuntu_gettext_domain,
                    locales_filter.as_deref(),
                    &mut unknown_keys,
                )?;
            }

            // insert keys which have no group
            for unknown_key in unknown_keys.drain(..) {
                match &mut active_group {
                    Some(active_group) => match active_group.group.0.get_mut(unknown_key.key) {
                        Some((_, locale_map)) => {
                            locale_map.insert(unknown_key.locale, unknown_key.value);
                        }
                        None => return Err(DecodeError::KeyDoesNotExist),
                    },
                    None => return Err(DecodeError::KeyDoesNotExist),
                }
            }

            if let Some(active_keys) = active_keys.take() {
                match &mut active_group {
                    Some(active_group) => {
                        active_group.group.0.insert(
                            active_keys.key_name,
                            (active_keys.default_value, active_keys.locales),
                        );
                    }
                    None => return Err(DecodeError::KeyValueWithoutAGroup),
                }
            }

            if let Some(mut group) = active_group.take() {
                groups
                    .0
                    .entry(group.group_name)
                    .or_insert_with(|| Group::default())
                    .0
                    .append(&mut group.group.0);
            }

            Ok(DesktopEntry {
                appid,
                groups,
                path,
                ubuntu_gettext_domain,
            })
        }

        inner(path.into(), input, locales_filter.map(add_generic_locales))
    }

    /// Return an owned [`DesktopEntry`]
    #[inline]
    pub fn from_path<L>(
        path: impl Into<PathBuf>,
        locales_filter: Option<&[L]>,
    ) -> Result<DesktopEntry, DecodeError>
    where
        L: AsRef<str>,
    {
        let path: PathBuf = path.into();
        let input = fs::read_to_string(&path)?;
        Self::from_str(path, &input, locales_filter)
    }
}

#[inline]
fn get_app_id<P: AsRef<Path> + ?Sized>(path: &P) -> Result<String, DecodeError> {
    let path_as_bytes = path
        .as_ref()
        .as_os_str()
        .as_encoded_bytes()
        .strip_suffix(b".desktop")
        .ok_or(DecodeError::AppID)?;

    Ok(
        if let Some((_prefix, entry)) = path_as_bytes.rsplit_once_str("/applications/") {
            String::from_utf8(entry.replace(b"/", b"-"))
                .ok()
                .ok_or(DecodeError::AppID)?
        } else {
            path.as_ref()
                .file_stem()
                .ok_or(DecodeError::AppID)?
                .to_str()
                .ok_or(DecodeError::AppID)?
                .to_owned()
        },
    )
}

#[derive(Debug)]
struct ActiveGroup {
    group_name: String,
    group: Group,
}

#[derive(Debug)]
struct ActiveKeys {
    key_name: String,
    default_value: String,
    locales: LocaleMap,
}

#[inline(never)]
fn process_line<'a>(
    line: &'a str,
    groups: &mut Groups,
    active_group: &mut Option<ActiveGroup>,
    active_keys: &mut Option<ActiveKeys>,
    ubuntu_gettext_domain: &mut Option<String>,
    locales_filter: Option<&[&str]>,
    unknown_keys: &mut Vec<UnknownKey<'a>>,
) -> Result<(), DecodeError> {
    if line.trim().is_empty() || line.starts_with('#') {
        return Ok(());
    }

    let line_bytes = line.as_bytes();

    // if group
    if line_bytes[0] == b'[' {
        // insert keys which have no group
        for unknown_key in unknown_keys.drain(..) {
            match active_group {
                Some(active_group) => match active_group.group.0.get_mut(unknown_key.key) {
                    Some((_, locale_map)) => {
                        locale_map.insert(unknown_key.locale, unknown_key.value);
                    }
                    None => return Err(DecodeError::KeyDoesNotExist),
                },
                None => return Err(DecodeError::KeyDoesNotExist),
            }
        }

        if let Some(end) = memchr::memrchr(b']', &line_bytes[1..]) {
            let group_name = &line[1..end + 1];

            if let Some(active_keys) = active_keys.take() {
                match active_group {
                    Some(active_group) => {
                        active_group.group.0.insert(
                            active_keys.key_name,
                            (active_keys.default_value, active_keys.locales),
                        );
                    }
                    None => return Err(DecodeError::KeyValueWithoutAGroup),
                }
            }

            if let Some(mut group) = active_group.take() {
                groups
                    .0
                    .entry(group.group_name)
                    .or_insert_with(|| Group::default())
                    .0
                    .append(&mut group.group.0);
            }

            active_group.replace(ActiveGroup {
                group_name: group_name.to_string(),
                group: Group::default(),
            });
        }
    }
    // else, if value
    else if let Some(delimiter) = memchr::memchr(b'=', line_bytes) {
        let key = &line[..delimiter];
        let value = format_value(&line[delimiter + 1..])?;

        if key.is_empty() {
            return Err(DecodeError::InvalidKey);
        }

        // if locale
        if key.as_bytes()[key.len() - 1] == b']' {
            if let Some(start) = memchr::memchr(b'[', key.as_bytes()) {
                let locale = &key[start + 1..key.len() - 1];

                let key = &key[..start];

                match locales_filter {
                    Some(locales_filter) if !locales_filter.iter().any(|l| *l == locale) => {
                        return Ok(());
                    }
                    _ => (),
                }

                // we verify that the name is the same of active key
                // even tho this is forbidden by the spec, nautilus does this for example
                if let Some(active_keys) = active_keys
                    .as_mut()
                    .filter(|active_keys| active_keys.key_name == key)
                {
                    active_keys.locales.insert(locale.to_string(), value);
                } else {
                    unknown_keys.push(UnknownKey {
                        key,
                        locale: locale.to_string(),
                        value,
                    });
                }

                return Ok(());
            }
        }

        if key == "X-Ubuntu-Gettext-Domain" {
            *ubuntu_gettext_domain = Some(value.to_string());
            return Ok(());
        }

        if let Some(active_keys) = active_keys.take() {
            match active_group {
                Some(active_group) => {
                    active_group.group.0.insert(
                        active_keys.key_name,
                        (active_keys.default_value, active_keys.locales),
                    );
                }
                None => return Err(DecodeError::KeyValueWithoutAGroup),
            }
        }
        active_keys.replace(ActiveKeys {
            // todo: verify that the key only contains A-Za-z0-9 ?
            key_name: key.trim().to_string(),
            default_value: value,
            locales: LocaleMap::default(),
        });
    }
    Ok(())
}

// https://specifications.freedesktop.org/desktop-entry-spec/latest/value-types.html
#[inline]
fn format_value(input: &str) -> Result<String, DecodeError> {
    let input = if let Some(input) = input.strip_prefix(" ") {
        input
    } else {
        input
    };

    let mut res = String::with_capacity(input.len());

    let mut last: usize = 0;

    for i in memchr::memchr_iter(b'\\', input.as_bytes()) {
        // edge case for //
        if last > i {
            continue;
        }

        // when there is an \ at the end
        if input.len() <= i + 1 {
            return Err(DecodeError::InvalidValue);
        }

        if last < i {
            res.push_str(&input[last..i]);
        }

        last = i + 2;

        match input.as_bytes()[i + 1] {
            b's' => res.push(' '),
            b'n' => res.push('\n'),
            b't' => res.push('\t'),
            b'r' => res.push('\r'),
            b'\\' => res.push('\\'),
            _ => {
                return Err(DecodeError::InvalidValue);
            }
        }
    }

    if last < input.len() {
        res.push_str(&input[last..input.len()]);
    }

    Ok(res)
}

/// Ex: if a locale equal fr_FR, add fr
#[inline]
fn add_generic_locales<L: AsRef<str>>(locales: &[L]) -> Vec<&str> {
    let mut v = Vec::with_capacity(locales.len() + 1);

    for l in locales {
        let l = l.as_ref();

        v.push(l);

        if let Some(start) = memchr::memchr(b'_', l.as_bytes()) {
            v.push(l.split_at(start).0)
        }
    }

    v
}
