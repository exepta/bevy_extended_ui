#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
use crate::dialog::{DialogClosed, DialogConfig, DialogProvider, DialogResult, ShowDialog};
use crate::styles::{CssSource, TagName};
use crate::widgets::{HyperLink, HyperLinkBrowsers, UIGenID, UIWidgetState, WidgetId, WidgetKind};
use crate::{CurrentWidgetState, ExtendedUiConfiguration};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
use std::fs;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::process::{Command, Stdio};
#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
use web_sys::window;

/// Marker component for initialized hyperlink widgets.
#[derive(Component)]
struct HyperLinkBase;

#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
#[derive(Resource, Default)]
struct HyperLinkInstallRequests {
    next_request_id: u64,
    pending_commands: HashMap<u64, String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BrowserAttempt {
    Opened,
    NotInstalled,
    LaunchFailed,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Eq, PartialEq)]
enum HyperLinkOpenOutcome {
    Opened,
    MissingBrowser { requested: Vec<String> },
    Failed,
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum LinuxFamily {
    Arch,
    Debian,
    Fedora,
}

/// Plugin that registers hyperlink widget behavior.
pub struct HyperLinkWidget;

impl Plugin for HyperLinkWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (internal_node_creation_system, update_text));

        #[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
        {
            app.init_resource::<HyperLinkInstallRequests>();
            app.add_systems(Update, handle_install_dialog_closed);
        }
    }
}

/// Initializes internal UI nodes for hyperlinks.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &HyperLink, Option<&CssSource>),
        (With<HyperLink>, Without<HyperLinkBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = config.render_layers.first().copied().unwrap_or(1);

    for (entity, link, source_opt) in query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands
            .entity(entity)
            .insert((
                Name::new(format!("HyperLink-{}", link.entry)),
                Node::default(),
                WidgetId {
                    id: link.entry,
                    kind: WidgetKind::HyperLink,
                },
                Text::new(link.text.clone()),
                TextColor::default(),
                TextFont::default(),
                TextLayout::default(),
                ZIndex::default(),
                Pickable::default(),
                css_source,
                TagName("a".to_string()),
                RenderLayers::layer(layer),
                HyperLinkBase,
            ))
            .observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave);
    }
}

/// Synchronizes visible text with `HyperLink::text`.
fn update_text(mut query: Query<(&mut Text, &HyperLink), With<HyperLink>>) {
    for (mut text, link) in query.iter_mut() {
        text.0 = link.text.clone();
    }
}

/// Handles hyperlink click events.
fn on_internal_click(
    mut trigger: On<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID, &HyperLink), With<HyperLink>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
    mut show_dialog: MessageWriter<ShowDialog>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
    mut install_requests: ResMut<HyperLinkInstallRequests>,
) {
    if let Ok((mut state, gen_id, link)) = query.get_mut(trigger.entity) {
        state.focused = true;
        current_widget_state.widget_id = gen_id.get();
        let link = link.clone();

        #[cfg(target_arch = "wasm32")]
        {
            open_hyper_link_wasm(&link);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let outcome = open_hyper_link_native(&link);

            #[cfg(feature = "extended-dialog")]
            if let HyperLinkOpenOutcome::MissingBrowser { requested } = outcome {
                let requested_name = requested
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "browser".to_string());

                if link.open_modal {
                    if let Some(install_command) =
                        build_install_command_for_browser(&requested_name)
                    {
                        let content = format!(
                            "The configured browser '{}' is not installed.\n\nRun this command in a terminal?\n{}",
                            requested_name, install_command
                        );

                        let request_id = next_install_request_id(&mut install_requests);
                        install_requests
                            .pending_commands
                            .insert(request_id, install_command);

                        let mut config = DialogConfig::question("Install Browser", content);
                        config.provider = DialogProvider::BevyApp;
                        show_dialog.write(ShowDialog {
                            request_id: Some(request_id),
                            config,
                        });
                    } else {
                        let mut config = DialogConfig::default_modal(
                            "Browser Not Installed",
                            format!(
                                "The configured browser '{}' is not installed. No supported install command is available for this operating system.",
                                requested_name
                            ),
                        );
                        config.provider = DialogProvider::BevyApp;
                        show_dialog.write(ShowDialog::new(config));
                    }
                } else {
                    let mut config = DialogConfig::default_modal(
                        "Browser Not Installed",
                        format!(
                            "The configured browser '{}' is not installed.",
                            requested_name
                        ),
                    );
                    config.provider = DialogProvider::BevyApp;
                    show_dialog.write(ShowDialog::new(config));
                }
            }

            #[cfg(not(feature = "extended-dialog"))]
            if let HyperLinkOpenOutcome::MissingBrowser { requested, .. } = outcome {
                warn!(
                    "Configured browser(s) not installed: {:?}. Enable feature `extended-dialog` for modal prompts.",
                    requested
                );
            }
        }
    }

    trigger.propagate(false);
}

/// Sets hover state to true for hyperlinks.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<HyperLink>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }

    trigger.propagate(false);
}

/// Sets hover state to false for hyperlinks.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<HyperLink>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }

    trigger.propagate(false);
}

#[cfg(target_arch = "wasm32")]
fn open_hyper_link_wasm(link: &HyperLink) {
    let href = link.href.trim();
    if href.is_empty() {
        warn!("HyperLink click ignored: missing href.");
        return;
    }

    if !matches!(link.browsers, HyperLinkBrowsers::System) {
        warn!("HyperLink `browsers` attribute is ignored on wasm32; using system browser.");
    }

    #[cfg(feature = "clipboard-wasm")]
    {
        if let Some(win) = window() {
            if win.open_with_url(href).is_err() {
                warn!("Failed to open hyperlink in wasm window: {}", href);
            }
        } else {
            warn!("Unable to access browser window for hyperlink: {}", href);
        }
    }

    #[cfg(not(feature = "clipboard-wasm"))]
    {
        warn!("HyperLink opening on wasm32 requires feature `clipboard-wasm`.");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_hyper_link_native(link: &HyperLink) -> HyperLinkOpenOutcome {
    let href = link.href.trim();
    if href.is_empty() {
        warn!("HyperLink click ignored: missing href.");
        return HyperLinkOpenOutcome::Failed;
    }

    match &link.browsers {
        HyperLinkBrowsers::System => {
            if open_with_system_browser(href) {
                HyperLinkOpenOutcome::Opened
            } else {
                warn!("Failed to open hyperlink with system browser: {}", href);
                HyperLinkOpenOutcome::Failed
            }
        }
        HyperLinkBrowsers::Custom(requested) => {
            let mut any_installed = false;
            for browser in requested {
                match try_open_specific_browser(browser, href) {
                    BrowserAttempt::Opened => return HyperLinkOpenOutcome::Opened,
                    BrowserAttempt::LaunchFailed => any_installed = true,
                    BrowserAttempt::NotInstalled => {}
                }
            }

            if any_installed {
                warn!(
                    "HyperLink browser was installed but failed to launch: {:?}",
                    requested
                );
                HyperLinkOpenOutcome::Failed
            } else {
                HyperLinkOpenOutcome::MissingBrowser {
                    requested: requested.clone(),
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_with_system_browser(href: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", href]);
        return spawn_silent(&mut command);
    }

    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        command.arg(href);
        return spawn_silent(&mut command);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut command = Command::new("xdg-open");
        command.arg(href);
        return spawn_silent(&mut command);
    }

    #[allow(unreachable_code)]
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn try_open_specific_browser(browser: &str, href: &str) -> BrowserAttempt {
    #[cfg(target_os = "windows")]
    {
        return try_open_browser_windows(browser, href);
    }

    #[cfg(target_os = "macos")]
    {
        return try_open_browser_macos(browser, href);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return try_open_browser_linux(browser, href);
    }

    #[allow(unreachable_code)]
    BrowserAttempt::NotInstalled
}

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
fn try_open_browser_linux(browser: &str, href: &str) -> BrowserAttempt {
    let candidates = browser_binary_candidates(browser);
    for candidate in candidates {
        if command_exists(&candidate) {
            let mut command = Command::new(&candidate);
            command.arg(href);
            return if spawn_silent(&mut command) {
                BrowserAttempt::Opened
            } else {
                BrowserAttempt::LaunchFailed
            };
        }
    }
    BrowserAttempt::NotInstalled
}

#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
fn try_open_browser_macos(browser: &str, href: &str) -> BrowserAttempt {
    let candidates = browser_app_candidates(browser);
    for app_name in candidates {
        let installed = Command::new("open")
            .args(["-Ra", app_name.as_str()])
            .status()
            .ok()
            .is_some_and(|status| status.success());
        if installed {
            return if Command::new("open")
                .args(["-a", app_name.as_str(), href])
                .spawn()
                .is_ok()
            {
                BrowserAttempt::Opened
            } else {
                BrowserAttempt::LaunchFailed
            };
        }
    }
    BrowserAttempt::NotInstalled
}

#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
fn try_open_browser_windows(browser: &str, href: &str) -> BrowserAttempt {
    for candidate in browser_windows_candidates(browser) {
        if candidate.is_file() {
            let mut command = Command::new(candidate);
            command.arg(href);
            return if spawn_silent(&mut command) {
                BrowserAttempt::Opened
            } else {
                BrowserAttempt::LaunchFailed
            };
        }
    }

    for candidate in browser_binary_candidates(browser) {
        if command_exists(&candidate) {
            let mut command = Command::new(&candidate);
            command.arg(href);
            return if spawn_silent(&mut command) {
                BrowserAttempt::Opened
            } else {
                BrowserAttempt::LaunchFailed
            };
        }
    }

    BrowserAttempt::NotInstalled
}

#[cfg(not(target_arch = "wasm32"))]
fn canonical_browser_name(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    match normalized.as_str() {
        "google-chrome" | "chrome" => "chrome".to_string(),
        "mozilla-firefox" | "firefox" => "firefox".to_string(),
        "brave-browser" | "brave" => "brave".to_string(),
        "microsoft-edge" | "msedge" | "edge" => "edge".to_string(),
        "chromium-browser" | "chromium" => "chromium".to_string(),
        other => other.to_string(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_binary_candidates(raw_browser: &str) -> Vec<String> {
    let browser = canonical_browser_name(raw_browser);
    match browser.as_str() {
        "firefox" => vec!["firefox".to_string()],
        "chrome" => vec![
            "google-chrome".to_string(),
            "google-chrome-stable".to_string(),
            "chrome".to_string(),
        ],
        "brave" => vec!["brave-browser".to_string(), "brave".to_string()],
        "edge" => vec![
            "microsoft-edge".to_string(),
            "microsoft-edge-stable".to_string(),
            "msedge".to_string(),
        ],
        "chromium" => vec!["chromium".to_string(), "chromium-browser".to_string()],
        "opera" => vec!["opera".to_string()],
        "vivaldi" => vec!["vivaldi".to_string()],
        _ => vec![browser],
    }
}

#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
fn browser_app_candidates(raw_browser: &str) -> Vec<String> {
    let browser = canonical_browser_name(raw_browser);
    match browser.as_str() {
        "firefox" => vec!["Firefox".to_string()],
        "chrome" => vec!["Google Chrome".to_string()],
        "brave" => vec!["Brave Browser".to_string()],
        "edge" => vec!["Microsoft Edge".to_string()],
        "chromium" => vec!["Chromium".to_string()],
        "safari" => vec!["Safari".to_string()],
        "opera" => vec!["Opera".to_string()],
        "vivaldi" => vec!["Vivaldi".to_string()],
        _ => vec![raw_browser.to_string()],
    }
}

#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
fn browser_windows_candidates(raw_browser: &str) -> Vec<PathBuf> {
    let browser = canonical_browser_name(raw_browser);
    let mut out = Vec::new();

    let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    let local_app_data = env::var_os("LocalAppData").map(PathBuf::from);

    let mut push_candidates = |relative: &str| {
        if let Some(base) = program_files.as_ref() {
            out.push(base.join(relative));
        }
        if let Some(base) = program_files_x86.as_ref() {
            out.push(base.join(relative));
        }
        if let Some(base) = local_app_data.as_ref() {
            out.push(base.join(relative));
        }
    };

    match browser.as_str() {
        "firefox" => push_candidates("Mozilla Firefox\\firefox.exe"),
        "chrome" => push_candidates("Google\\Chrome\\Application\\chrome.exe"),
        "brave" => push_candidates("BraveSoftware\\Brave-Browser\\Application\\brave.exe"),
        "edge" => push_candidates("Microsoft\\Edge\\Application\\msedge.exe"),
        "opera" => push_candidates("Programs\\Opera\\opera.exe"),
        "vivaldi" => push_candidates("Vivaldi\\Application\\vivaldi.exe"),
        "chromium" => push_candidates("Chromium\\Application\\chrome.exe"),
        _ => {}
    }

    out
}

#[cfg(not(target_arch = "wasm32"))]
fn command_exists(command: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    #[cfg(target_os = "windows")]
    {
        let has_extension = command.contains('.');
        let extensions = if has_extension {
            vec!["".to_string()]
        } else {
            env::var_os("PATHEXT")
                .map(|raw| {
                    raw.to_string_lossy()
                        .split(';')
                        .filter(|entry| !entry.trim().is_empty())
                        .map(|entry| entry.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .filter(|exts| !exts.is_empty())
                .unwrap_or_else(|| vec![".EXE".to_string(), ".CMD".to_string(), ".BAT".to_string()])
        };

        for dir in env::split_paths(&path_var) {
            for ext in &extensions {
                let candidate = if ext.is_empty() {
                    dir.join(command)
                } else {
                    dir.join(format!("{command}{ext}"))
                };
                if candidate.is_file() {
                    return true;
                }
            }
        }

        return false;
    }

    #[cfg(not(target_os = "windows"))]
    {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(command);
            if candidate.is_file() {
                return true;
            }
        }

        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_silent(command: &mut Command) -> bool {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
fn next_install_request_id(requests: &mut HyperLinkInstallRequests) -> u64 {
    requests.next_request_id = requests.next_request_id.saturating_add(1);
    requests.next_request_id
}

#[cfg(all(not(target_arch = "wasm32"), feature = "extended-dialog"))]
fn handle_install_dialog_closed(
    mut dialogs_closed: MessageReader<DialogClosed>,
    mut install_requests: ResMut<HyperLinkInstallRequests>,
) {
    for event in dialogs_closed.read() {
        let Some(command) = install_requests.pending_commands.remove(&event.request_id) else {
            continue;
        };

        if event.provider != DialogProvider::BevyApp {
            continue;
        }

        if event.result == DialogResult::Confirmed && !open_install_command_in_terminal(&command) {
            warn!("Failed to open terminal for install command: {}", command);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_install_command_for_browser(browser: &str) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let family = detect_linux_family()?;
        let package = linux_package_for_browser(browser, family);
        let command = match family {
            LinuxFamily::Arch => format!("sudo pacman -S {package}"),
            LinuxFamily::Debian => format!("sudo apt install {package}"),
            LinuxFamily::Fedora => format!("sudo dnf install {package}"),
        };
        return Some(command);
    }

    #[cfg(target_os = "macos")]
    {
        let package = brew_package_for_browser(browser);
        return Some(format!("brew install --cask {package}"));
    }

    #[cfg(target_os = "windows")]
    {
        let package = winget_package_for_browser(browser);
        return Some(format!("winget install --id {package} -e"));
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = browser;
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_install_command_in_terminal(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        return Command::new("cmd")
            .args(["/C", "start", "", "cmd", "/K", command])
            .spawn()
            .is_ok();
    }

    #[cfg(target_os = "macos")]
    {
        let escaped = escape_applescript_text(command);
        return Command::new("osascript")
            .args([
                "-e",
                format!("tell application \"Terminal\" to do script \"{escaped}\"").as_str(),
                "-e",
                "tell application \"Terminal\" to activate",
            ])
            .spawn()
            .is_ok();
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return open_linux_terminal_with_command(command);
    }

    #[allow(unreachable_code)]
    false
}

#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
fn escape_applescript_text(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn open_linux_terminal_with_command(command: &str) -> bool {
    let interactive_command = format!(r#"{command}; echo; read -r -p "Press Enter to close..." _"#);

    if command_exists("x-terminal-emulator")
        && Command::new("x-terminal-emulator")
            .args(["-e", "bash", "-lc", interactive_command.as_str()])
            .spawn()
            .is_ok()
    {
        return true;
    }

    if command_exists("gnome-terminal")
        && Command::new("gnome-terminal")
            .args(["--", "bash", "-lc", interactive_command.as_str()])
            .spawn()
            .is_ok()
    {
        return true;
    }

    if command_exists("konsole")
        && Command::new("konsole")
            .args(["-e", "bash", "-lc", interactive_command.as_str()])
            .spawn()
            .is_ok()
    {
        return true;
    }

    if command_exists("alacritty")
        && Command::new("alacritty")
            .args(["-e", "bash", "-lc", interactive_command.as_str()])
            .spawn()
            .is_ok()
    {
        return true;
    }

    if command_exists("kitty")
        && Command::new("kitty")
            .args(["-e", "bash", "-lc", interactive_command.as_str()])
            .spawn()
            .is_ok()
    {
        return true;
    }

    if command_exists("xterm")
        && Command::new("xterm")
            .args(["-e", "bash", "-lc", interactive_command.as_str()])
            .spawn()
            .is_ok()
    {
        return true;
    }

    false
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn detect_linux_family() -> Option<LinuxFamily> {
    let raw = fs::read_to_string("/etc/os-release").ok()?;
    let id = parse_os_release_value(&raw, "ID").unwrap_or_default();
    let id_like = parse_os_release_value(&raw, "ID_LIKE").unwrap_or_default();
    let combined = format!("{} {}", id, id_like).to_ascii_lowercase();

    if combined.contains("arch") || combined.contains("manjaro") {
        return Some(LinuxFamily::Arch);
    }
    if combined.contains("debian")
        || combined.contains("ubuntu")
        || combined.contains("mint")
        || combined.contains("pop")
    {
        return Some(LinuxFamily::Debian);
    }
    if combined.contains("fedora")
        || combined.contains("rhel")
        || combined.contains("centos")
        || combined.contains("rocky")
    {
        return Some(LinuxFamily::Fedora);
    }

    None
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn parse_os_release_value(raw: &str, key: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((entry_key, value)) = trimmed.split_once('=') else {
            continue;
        };

        if entry_key.trim() != key {
            continue;
        }

        let parsed = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_ascii_lowercase();
        return Some(parsed);
    }

    None
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn linux_package_for_browser(browser: &str, family: LinuxFamily) -> String {
    let normalized = canonical_browser_name(browser);
    match (normalized.as_str(), family) {
        ("firefox", LinuxFamily::Debian) => "firefox-esr".to_string(),
        ("firefox", _) => "firefox".to_string(),
        ("chrome", LinuxFamily::Arch) => "google-chrome".to_string(),
        ("chrome", _) => "google-chrome-stable".to_string(),
        ("brave", _) => "brave-browser".to_string(),
        ("edge", _) => "microsoft-edge-stable".to_string(),
        ("chromium", _) => "chromium".to_string(),
        ("opera", _) => "opera".to_string(),
        ("vivaldi", _) => "vivaldi".to_string(),
        _ => normalized,
    }
}

#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
fn brew_package_for_browser(browser: &str) -> String {
    let normalized = canonical_browser_name(browser);
    match normalized.as_str() {
        "firefox" => "firefox".to_string(),
        "chrome" => "google-chrome".to_string(),
        "brave" => "brave-browser".to_string(),
        "edge" => "microsoft-edge".to_string(),
        "chromium" => "chromium".to_string(),
        "opera" => "opera".to_string(),
        "vivaldi" => "vivaldi".to_string(),
        _ => normalized,
    }
}

#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
fn winget_package_for_browser(browser: &str) -> String {
    let normalized = canonical_browser_name(browser);
    match normalized.as_str() {
        "firefox" => "Mozilla.Firefox".to_string(),
        "chrome" => "Google.Chrome".to_string(),
        "brave" => "Brave.Brave".to_string(),
        "edge" => "Microsoft.Edge".to_string(),
        "chromium" => "Chromium.Chromium".to_string(),
        "opera" => "Opera.Opera".to_string(),
        "vivaldi" => "VivaldiTechnologies.Vivaldi".to_string(),
        _ => normalized,
    }
}
