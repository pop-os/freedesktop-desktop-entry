// Copyright 2021 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::exec::error::ExecError;
use crate::exec::graphics::Gpus;
use crate::DesktopEntry;
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::dbus_proxy;
use zbus::names::OwnedBusName;
use zbus::zvariant::{OwnedValue, Str};

#[dbus_proxy(interface = "org.freedesktop.Application")]
trait Application {
    fn activate(&self, platform_data: HashMap<String, OwnedValue>) -> zbus::Result<()>;
    fn activate_action(
        &self,
        action_name: &str,
        parameters: &[OwnedValue],
        platform_data: HashMap<String, OwnedValue>,
    ) -> zbus::Result<()>;
    fn open(&self, uris: &[&str], platform_data: HashMap<String, OwnedValue>) -> zbus::Result<()>;
}

impl DesktopEntry<'_> {
    pub(crate) fn dbus_launch(
        &self,
        conn: &Connection,
        uris: &[&str],
    ) -> Result<(), ExecError> {
        let dbus_path = self.appid.replace('.', "/");
        let dbus_path = format!("/{dbus_path}");
        let app_proxy = ApplicationProxyBlocking::builder(conn)
            .destination(self.appid)?
            .path(dbus_path.as_str())?
            .build()?;

        let mut platform_data = HashMap::new();
        if self.prefers_non_default_gpu() {
            let gpus = Gpus::load();
            if let Some(gpu) = gpus.non_default() {
                for (opt, value) in gpu.launch_options() {
                    platform_data.insert(opt, OwnedValue::from(Str::from(value.as_str())));
                }
            }
        }

        if !uris.is_empty() {
            app_proxy.open(uris, platform_data)?;
        } else {
            app_proxy.activate(platform_data)?;
        }

        Ok(())
    }

    pub(crate) fn is_bus_actionable(&self, conn: &Connection) -> bool {
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
}
