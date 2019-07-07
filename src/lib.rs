//! # Freedesktop Desktop Entry Specification
//!
//! This crate provides a library for efficiently generating valid desktop entries.
//!
//! - [Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/index.html)
//!
//! ```
//! use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
//! use std::{
//!     env,
//!     fs::File,
//!     io::Write,
//!     path::{Path, PathBuf},
//! };
//!
//! const APPID: &str = "com.system76.Popsicle";
//!
//! fn main() {
//!     let exec_path = Path::new("/usr").join("bin").join(APPID);
//!     let exec = exec_path.as_os_str().to_str().expect("prefix is not UTF-8");
//!
//!     let mut desktop = File::create(["target/", APPID, ".desktop"].concat().as_str())
//!         .expect("failed to create desktop entry file");
//!
//!     let entry = DesktopEntry::new(
//!         "Popsicle",
//!         APPID,
//!         DesktopType::Application(
//!             Application::new("System", exec)
//!                 .keywords(&["usb", "flash" ,"drive", "image"])
//!                 .startup_notify(),
//!         ),
//!     )
//!     .comment("Multiple USB image flasher")
//!     .generic_name("USB Flasher");
//!
//!     desktop.write_all(entry.to_string().as_bytes());
//! }
//! ```

pub struct Application<'a> {
    pub categories: &'a str,
    pub exec: &'a str,
    pub keywords: &'a [&'a str],
    pub mime_types: &'a [&'a str],
    pub path: Option<&'a str>,
    pub startup_notify: bool,
    pub startup_wm_class: Option<&'a str>,
    pub terminal: bool,
    pub try_exec: Option<&'a str>,
}

impl<'a> Application<'a> {
    pub fn new(categories: &'a str, exec: &'a str) -> Self {
        Application {
            categories,
            exec,
            keywords: &[],
            mime_types: &[],
            path: None,
            startup_notify: false,
            startup_wm_class: None,
            terminal: false,
            try_exec: None,
        }
    }

    pub fn keywords(mut self, keywords: &'a [&'a str]) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn mime_types(mut self, mime_types: &'a [&'a str]) -> Self {
        self.mime_types = mime_types;
        self
    }

    pub fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    pub fn startup_notify(mut self) -> Self {
        self.startup_notify = true;
        self
    }

    pub fn startup_wm_class(mut self, startup_wm_class: &'a str) -> Self {
        self.startup_wm_class = Some(startup_wm_class);
        self
    }

    pub fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }

    pub fn try_exec(mut self, try_exec: &'a str) -> Self {
        self.try_exec = Some(try_exec);
        self
    }
}

pub enum DesktopType<'a> {
    Application(Application<'a>),
    Directory,
    Link { url: &'a str },
}

impl<'a> DesktopType<'a> {
    pub fn type_str(&self) -> &'static str {
        match self {
            DesktopType::Application(_) => "Application",
            DesktopType::Directory => "Directory",
            DesktopType::Link { .. } => "Link",
        }
    }
}

markup::define! {
    DesktopEntry<'a>(
        name: &'a str,
        generic_name: Option<&'a str>,
        icon: &'a str,
        comment: Option<&'a str>,
        hidden: bool,
        no_display: bool,
        kind: DesktopType<'a>
    ) {
        "[Desktop Entry]\n"
        "Type=" {markup::raw(kind.type_str())} "\n"
        "Name=" {markup::raw(name)} "\n"

        @if let Some(generic) = (generic_name) {
            "GenericName=" {markup::raw(generic)} "\n"

            "X-GNOME-FullName=" {markup::raw(name)} " " {markup::raw(generic)} "\n"
        }

        "Icon=" {icon} "\n"

        @if let Some(comment) = (comment) {
            "Comment=" {markup::raw(comment)} "\n"
        }

        @if *(hidden) {
            "Hidden=true\n"
        }

        @if *(no_display) {
            "NoDisplay=true\n"
        }

        @if let DesktopType::Application(app) = (kind) {
            "Categories=" {markup::raw(app.categories)} "\n"

            @if !app.keywords.is_empty() {
                "Keywords=" { '\"' }
                @for keyword in app.keywords.iter() {
                    {markup::raw(keyword)} ";"
                }
                { markup::raw("\"\n") }
            }

            @if !app.mime_types.is_empty() {
                "MimeType=" { '\"' }
                @for mime in app.mime_types {
                    {markup::raw(mime)} ";"
                }
                { markup::raw("\"\n") }
            }

            @if app.terminal {
                "Terminal=true\n"
            }

            @if app.startup_notify {
                "StartupNotify=true\n"
            }

            "Exec=" {markup::raw(app.exec)} "\n"

            @if let Some(path) = (app.path) {
                "Path=" {markup::raw(path)} "\n"
            }
        } else if let DesktopType::Link { url } = (kind) {
            "Link=" {markup::raw(url)} "\n"
        }
    }
}

impl<'a> DesktopEntry<'a> {
    pub fn new(name: &'a str, icon: &'a str, kind: DesktopType<'a>) -> Self {
        DesktopEntry {
            name,
            generic_name: None,
            icon,
            kind,
            comment: None,
            hidden: false,
            no_display: false,
        }
    }

    pub fn generic_name(mut self, name: &'a str) -> Self {
        self.generic_name = Some(name);
        self
    }

    pub fn comment(mut self, comment: &'a str) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn no_display(mut self) -> Self {
        self.no_display = true;
        self
    }
}
