// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::exec::error::ExecError;
use crate::exec::graphics::Gpus;
use crate::DesktopEntry;
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::names::OwnedBusName;
use zbus::proxy;
use zbus::zvariant::{OwnedValue, Str};

// https://specifications.freedesktop.org/desktop-entry-spec/1.1/ar01s07.html
#[proxy(interface = "org.freedesktop.Application")]
trait Application {
    fn activate(&self, platform_data: HashMap<String, OwnedValue>) -> zbus::Result<()>;

    fn open(&self, uris: &[&str], platform_data: HashMap<String, OwnedValue>) -> zbus::Result<()>;

    // XXX: https://gitlab.freedesktop.org/xdg/xdg-specs/-/issues/134
    fn activate_action(
        &self,
        action_name: &str,
        parameters: &[&str],
        platform_data: HashMap<String, OwnedValue>,
    ) -> zbus::Result<()>;
}

impl DesktopEntry<'_> {
    pub(crate) fn should_launch_on_dbus(&self) -> Option<Connection> {
        match self.desktop_entry_bool("DBusActivatable") {
            true => match Connection::session() {
                Ok(conn) => {
                    if self.is_bus_actionable(&conn) {
                        Some(conn)
                    } else {
                        None
                    }
                }
                Err(e) => {
                    log::error!("can't open dbus session: {}", e);
                    None
                }
            },
            false => None,
        }
    }

    fn is_bus_actionable(&self, conn: &Connection) -> bool {
        let dbus_proxy = zbus::blocking::fdo::DBusProxy::new(conn);

        if dbus_proxy.is_err() {
            return false;
        }

        let dbus_proxy = dbus_proxy.unwrap();
        let dbus_names = dbus_proxy.list_activatable_names();

        if dbus_names.is_err() {
            return false;
        }

        let dbus_names = dbus_names.unwrap();

        dbus_names
            .into_iter()
            .map(OwnedBusName::into_inner)
            .any(|name| name.as_str() == self.appid)
    }

    pub(crate) fn dbus_launch(&self, conn: &Connection, uris: &[&str]) -> Result<(), ExecError> {
        let app_proxy = self.get_app_proxy(conn)?;
        let platform_data = self.get_platform_data();

        if !uris.is_empty() {
            app_proxy.open(uris, platform_data)?;
        } else {
            app_proxy.activate(platform_data)?;
        }

        Ok(())
    }

    pub(crate) fn dbus_launch_action(
        &self,
        conn: &Connection,
        action_name: &str,
        uris: &[&str],
    ) -> Result<(), ExecError> {
        let app_proxy = self.get_app_proxy(conn)?;
        let platform_data = self.get_platform_data();
        app_proxy.activate_action(action_name, uris, platform_data)?;

        Ok(())
    }

    fn get_app_proxy(&self, conn: &Connection) -> Result<ApplicationProxyBlocking, ExecError> {
        let dbus_path = self.appid.replace('.', "/").replace('-', "_");
        let dbus_path = format!("/{dbus_path}");
        let app_proxy = ApplicationProxyBlocking::builder(conn)
            .destination(self.appid.as_ref())?
            .path(dbus_path)?
            .build()?;
        Ok(app_proxy)
    }

    // todo: XDG_ACTIVATION_TOKEN and DESKTOP_STARTUP_ID ?
    // https://github.com/pop-os/libcosmic/blob/master/src/app/mod.rs
    fn get_platform_data(&self) -> HashMap<String, OwnedValue> {
        let mut platform_data = HashMap::new();
        if self.prefers_non_default_gpu() {
            let gpus = Gpus::load();
            if let Some(gpu) = gpus.non_default() {
                for (opt, value) in gpu.launch_options() {
                    platform_data.insert(opt, OwnedValue::from(Str::from(value.as_str())));
                }
            }
        }
        platform_data
    }
}
