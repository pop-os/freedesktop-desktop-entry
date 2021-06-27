use std::borrow::Cow;
use std::collections::BTreeMap;

pub type MapStr = Cow<'static, str>;
pub type KeyMap = BTreeMap<MapStr, (MapStr, BTreeMap<MapStr, MapStr>)>;

pub struct DesktopEntryBuf {
    pub appid: MapStr,
    pub groups: BTreeMap<MapStr, KeyMap>,
}

impl DesktopEntryBuf {
    pub fn new(appid: MapStr) -> Self {
        Self {
            appid,
            groups: BTreeMap::new(),
        }
    }

    pub fn action_entry(&mut self, action: MapStr) -> &mut KeyMap {
        self.groups
            .entry(["Desktop Action ", &*action].concat().into())
            .or_insert_with(|| KeyMap::new())
    }

    pub fn action_exec(&mut self, action: MapStr, exec: MapStr) -> &mut Self {
        self.set_localized_action(action, "Exec".into(), exec, None)
    }

    pub fn action_name(
        &mut self,
        action: MapStr,
        name: MapStr,
        locale: Option<MapStr>,
    ) -> &mut Self {
        self.set_localized_action(action, "Name".into(), name, locale)
    }

    pub fn actions(&mut self, actions: MapStr) -> &mut Self {
        self.set_localized("Actions".into(), actions, None)
    }

    pub fn categories(&mut self, categories: MapStr) -> &mut Self {
        self.set_localized("Categories".into(), categories, None)
    }

    pub fn comment(&mut self, comment: MapStr, locale: Option<MapStr>) -> &mut Self {
        self.set_localized("Comment".into(), comment, locale)
    }

    pub fn desktop_entry(&mut self) -> &mut KeyMap {
        self.groups
            .entry("Desktop Entry".into())
            .or_insert_with(|| KeyMap::new())
    }

    pub fn exec(&mut self, exec: MapStr) -> &mut Self {
        self.set_localized("Exec".into(), exec, None)
    }

    pub fn icon(&mut self, icon: MapStr) -> &mut Self {
        self.set_localized("Icon".into(), icon, None)
    }

    pub fn keywords(&mut self, keywords: MapStr) -> &mut Self {
        self.set_localized("Keywords".into(), keywords, None)
    }

    pub fn mime_type(&mut self, mime_type: MapStr) -> &mut Self {
        self.set_localized("MimeType".into(), mime_type, None)
    }

    pub fn name(&mut self, name: MapStr, locale: Option<MapStr>) -> &mut Self {
        self.set_localized("Name".into(), name, locale)
    }

    pub fn no_display(&mut self, value: bool) -> &mut Self {
        self.set_bool("NoDisplay".into(), value)
    }

    pub fn prefers_non_default_gpu(&mut self, value: bool) -> &mut Self {
        self.set_bool("PrefersNonDefaultGPU".into(), value)
    }

    pub fn startup_notify(&mut self, value: bool) -> &mut Self {
        self.set_bool("StartupNotify".into(), value)
    }

    pub fn terminal(&mut self, value: bool) -> &mut Self {
        self.set_bool("Terminal".into(), value)
    }

    pub fn type_(&mut self, type_: MapStr) -> &mut Self {
        self.set_localized("Type".into(), type_, None)
    }

    pub fn set_bool(&mut self, key: MapStr, value: bool) -> &mut Self {
        self.desktop_entry()
            .entry(key)
            .or_insert_with(|| ("".into(), BTreeMap::new()))
            .0 = MapStr::from(if value { "true" } else { "false" });

        self
    }

    pub fn set_localized(
        &mut self,
        key: MapStr,
        value: MapStr,
        locale: Option<MapStr>,
    ) -> &mut Self {
        let entry = self
            .desktop_entry()
            .entry(key)
            .or_insert_with(|| ("".into(), BTreeMap::new()));

        match locale {
            Some(locale) => {
                entry.1.insert(locale, value);
            }

            None => entry.0 = value,
        }

        self
    }

    pub fn set_localized_action(
        &mut self,
        action: MapStr,
        key: MapStr,
        value: MapStr,
        locale: Option<MapStr>,
    ) -> &mut Self {
        let entry = self
            .action_entry(action)
            .entry(key)
            .or_insert_with(|| ("".into(), BTreeMap::new()));

        match locale {
            Some(locale) => {
                entry.1.insert(locale, value);
            }

            None => entry.0 = value,
        }

        self
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for DesktopEntryBuf {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for (group, keymap) in &self.groups {
            let _ = writeln!(formatter, "[{}]", group);

            for (key, (value, localizations)) in keymap {
                let _ = writeln!(formatter, "{}={}", key, value);
                for (locale, localized) in localizations {
                    let _ = writeln!(formatter, "{}[{}]={}", key, locale, localized);
                }
            }
        }

        Ok(())
    }
}