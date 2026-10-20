//! Project links: the Help menu, the About dialog and the start screen open these. This build is a
//! fork of storytold/photocraft; its links point at the fork's repository.

use serde_json::{Value, json};

use crate::PhotocraftApp;

pub const GITHUB: &str = "https://github.com/685ks/photocraft";
pub const ISSUES: &str = "https://github.com/685ks/photocraft/issues";
/// The project this build is a fork of (named in the About box).
pub const UPSTREAM: &str = "https://github.com/storytold/photocraft";
/// The About box's provenance line. A legal notice, shown in English in every UI language.
pub const FORK_NOTE: &str = "A fork of storytold/photocraft, not affiliated with or endorsed by ArtCraft.";
/// Shown in About when the Arthavan Projects launcher started the app (`ARTHAVAN_LAUNCHER=1`).
/// PhotoCraft has no updater of its own and makes no update or telemetry requests; the launcher
/// installs and updates it from the 685ks/photocraft releases.
pub const LAUNCHER_UPDATES_NOTE: &str = "Updates are managed by Arthavan Projects";

/// Whether the Arthavan Projects launcher started this process.
pub fn launcher_managed() -> bool {
    launcher_managed_from(std::env::var("ARTHAVAN_LAUNCHER").ok().as_deref())
}

fn launcher_managed_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim() == "1")
}

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
pub const COMMANDS: &[(&str, &str)] = &[("help.github", GITHUB), ("help.reportIssue", ISSUES)];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut PhotocraftApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// "GitHub · Report an Issue…" as links, centred. Clicks route through [`open`] (the
/// platform browser service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let links = [(tl!("GitHub"), GITHUB), (tl!("Report an Issue…"), ISSUES)];
    let font = egui::FontId::proportional(12.5);
    let sep = "  ·  ";
    let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
        + 2.0 * ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
    let mut clicked: Option<&str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(url);
            }
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, start-page and About links) must open in the browser.
        // They route through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        // Every Help-menu link id reaches the service with its URL.
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        // The direct open() helper (start-page and About links) also uses it.
        open(&mut app, &ctx, UPSTREAM);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(UPSTREAM));
    }

    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(help, ["help.github", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        for (id, _) in COMMANDS {
            assert!(crate::menus::is_live(id) && crate::menus::is_enabled(&app, id), "{id}");
        }
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        for (id, url) in [("help.github", "https://github.com/685ks/photocraft"), ("help.reportIssue", "https://github.com/685ks/photocraft/issues")] {
            let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
            assert_eq!(r["url"], url);
            assert_eq!(app.ui.status, format!("Opened {url}"));
        }
    }
}

#[cfg(test)]
mod rebrand_tests {
    use super::*;

    /// This fork carries no ArtCraft community or website links (house rebrand rule).
    #[test]
    fn no_link_points_at_artcraft_or_storyteller_services() {
        for (id, url) in COMMANDS {
            for banned in ["artcraft", "discord.gg", "storyteller.ai"] {
                assert!(!url.to_ascii_lowercase().contains(banned), "{id} -> {url}");
            }
        }
        assert!(COMMANDS.iter().all(|(id, _)| !id.contains("discord") && !id.contains("artcraft")));
    }

    #[test]
    fn launcher_flag_is_read_from_arthavan_launcher() {
        assert!(launcher_managed_from(Some("1")));
        assert!(!launcher_managed_from(None));
        assert!(!launcher_managed_from(Some("0")));
        assert!(!launcher_managed_from(Some("")));
    }
}
