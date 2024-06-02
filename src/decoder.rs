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
    pub fn decode_from_str(
        path: &'a Path,
        input: &'a str,
    ) -> Result<DesktopEntry<'a>, DecodeError> {
        let appid = get_app_id(path)?;

        let mut groups = Groups::new();
        let mut active_group = Cow::Borrowed("");
        let mut ubuntu_gettext_domain = None;

        for line in input.lines() {
            process_line(
                line,
                &mut groups,
                &mut active_group,
                &mut ubuntu_gettext_domain,
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

    /// Return an owned [`DesktopEntry`]
    pub fn decode_from_path<R>(path: PathBuf) -> Result<DesktopEntry<'static>, DecodeError> {
        let file = File::open(&path)?;

        let appid = get_app_id(&path)?;

        let mut groups = Groups::new();
        let mut active_group = Cow::Borrowed("");
        let mut ubuntu_gettext_domain = None;

        let mut reader = io::BufReader::new(file);
        let mut buf = String::new();

        while reader.read_line(&mut buf)? != 0 {
            process_line(
                &buf,
                &mut groups,
                &mut active_group,
                &mut ubuntu_gettext_domain,
                |s| Cow::Owned(s.to_owned()),
            )
        }

        Ok(DesktopEntry {
            appid: Cow::Owned(appid.to_owned()),
            groups,
            path: Cow::Owned(path),
            ubuntu_gettext_domain,
        })
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
fn process_line<'a, 'b, 'c: 'b + 'a, F>(
    line: &'a str,
    groups: &'b mut Groups<'c>,
    active_group: &'b mut Cow<'c, str>,
    ubuntu_gettext_domain: &'b mut Option<Cow<'c, str>>,
    convert_to_cow: F,
) where
    F: Fn(&'a str) -> Cow<'c, str>,
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

        if key.as_bytes()[key.len() - 1] == b']' {
            if let Some(start) = memchr::memchr(b'[', key.as_bytes()) {
                let key_name = &key[..start];
                let locale = &key[start + 1..key.len() - 1];
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
