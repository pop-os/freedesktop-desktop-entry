use std::{
    borrow::Cow,
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use crate::DesktopEntry;
use crate::{Groups, LocaleMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("path does not contain a valid app ID")]
    AppID,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<'a> DesktopEntry<'a> {
    pub fn decode_from_str<L>(
        path: &'a Path,
        input: &'a str,
        locales: &[L],
    ) -> Result<DesktopEntry<'a>, DecodeError>
    where
        L: AsRef<str>,
    {
        let appid = get_app_id(path)?;

        let mut groups = Groups::new();
        let mut active_group = Cow::Borrowed("");
        let mut ubuntu_gettext_domain = None;

        let locales = add_generic_locales(locales);

        for line in input.lines() {
            process_line(
                line,
                &mut groups,
                &mut active_group,
                &mut ubuntu_gettext_domain,
                &locales,
                Cow::Borrowed,
            )
        }

        Ok(DesktopEntry {
            appid: Cow::Borrowed(appid),
            groups,
            path: Cow::Borrowed(path),
            ubuntu_gettext_domain,
        })
    }

    pub fn decode_from_paths<'i, 'l: 'i, L>(
        paths: impl Iterator<Item = PathBuf> + 'i,
        locales: &'l [L],
    ) -> impl Iterator<Item = Result<DesktopEntry<'static>, DecodeError>> + 'i
    where
        L: AsRef<str>,
    {
        let mut buf = String::new();
        let locales = add_generic_locales(locales);

        paths.map(move |path| decode_from_path_with_buf(path, &locales, &mut buf))
    }

    /// Return an owned [`DesktopEntry`]
    pub fn decode_from_path<L>(
        path: PathBuf,
        locales: &[L],
    ) -> Result<DesktopEntry<'static>, DecodeError>
    where
        L: AsRef<str>,
    {
        let mut buf = String::new();
        let locales = add_generic_locales(locales);
        decode_from_path_with_buf(path, &locales, &mut buf)
    }
}

fn get_app_id<P: AsRef<Path> + ?Sized>(path: &P) -> Result<&str, DecodeError> {
    let appid = path
        .as_ref()
        .file_stem()
        .ok_or(DecodeError::AppID)?
        .to_str()
        .ok_or(DecodeError::AppID)?;
    Ok(appid)
}

#[inline]
fn process_line<'buf, 'local_ref, 'res: 'local_ref + 'buf, F, L>(
    line: &'buf str,
    groups: &'local_ref mut Groups<'res>,
    active_group: &'local_ref mut Cow<'res, str>,
    ubuntu_gettext_domain: &'local_ref mut Option<Cow<'res, str>>,
    locales_filter: &[L],
    convert_to_cow: F,
) where
    F: Fn(&'buf str) -> Cow<'res, str>,
    L: AsRef<str>,
{
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return;
    }

    let line_bytes = line.as_bytes();

    if line_bytes[0] == b'[' {
        if let Some(end) = memchr::memrchr(b']', &line_bytes[1..]) {
            *active_group = convert_to_cow(&line[1..end + 1]);
        }
    } else if let Some(delimiter) = memchr::memchr(b'=', line_bytes) {
        let key = &line[..delimiter];
        let value = &line[delimiter + 1..];

        // if locale
        if key.as_bytes()[key.len() - 1] == b']' {
            if let Some(start) = memchr::memchr(b'[', key.as_bytes()) {
                let key_name = &key[..start];
                let locale = &key[start + 1..key.len() - 1];

                if !locales_filter.iter().any(|l| l.as_ref() == locale) {
                    return;
                }

                groups
                    .entry(active_group.clone())
                    .or_default()
                    .entry(convert_to_cow(key_name))
                    .or_insert_with(|| (Cow::Borrowed(""), LocaleMap::new()))
                    .1
                    .insert(convert_to_cow(locale), convert_to_cow(value));

                return;
            }
        }

        if key == "X-Ubuntu-Gettext-Domain" {
            *ubuntu_gettext_domain = Some(convert_to_cow(value));
            return;
        }

        groups
            .entry(active_group.clone())
            .or_default()
            .entry(convert_to_cow(key))
            .or_insert_with(|| (Cow::Borrowed(""), BTreeMap::new()))
            .0 = convert_to_cow(value);
    }
}

#[inline]
fn decode_from_path_with_buf<L>(
    path: PathBuf,
    locales: &[L],
    buf: &mut String,
) -> Result<DesktopEntry<'static>, DecodeError>
where
    L: AsRef<str>,
{
    let file = File::open(&path)?;

    let appid = get_app_id(&path)?;

    let mut groups = Groups::new();
    let mut active_group = Cow::Borrowed("");
    let mut ubuntu_gettext_domain = None;

    let mut reader = io::BufReader::new(file);

    while reader.read_line(buf)? != 0 {
        process_line(
            buf,
            &mut groups,
            &mut active_group,
            &mut ubuntu_gettext_domain,
            locales,
            |s| Cow::Owned(s.to_owned()),
        );
        buf.clear();
    }

    Ok(DesktopEntry {
        appid: Cow::Owned(appid.to_owned()),
        groups,
        path: Cow::Owned(path),
        ubuntu_gettext_domain,
    })
}

/// Ex: if a locale equal fr_FR, add fr
fn add_generic_locales<'a, L: AsRef<str>>(locales: &'a [L]) -> Vec<&'a str> {
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
