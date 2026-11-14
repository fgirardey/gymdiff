//! The notification area icon (the menu bar on macOS) that keeps cooldown.lol
//! reachable while its window is closed: a click opens the window, the menu
//! offers Open cooldown.lol and Quit cooldown.lol
//! (`docs/Product/Features/Running in the background.md`, ADR-0013).

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::platform;

/// The icon; it disappears when dropped, so keep it for the whole run.
pub struct Tray {
    _icon: TrayIcon,
}

impl Tray {
    /// Builds the icon and routes its events: `open` on a click or on Open
    /// cooldown.lol, `quit` on Quit cooldown.lol. Both run on the thread that delivers
    /// the event, so they hand their work to the event loop themselves.
    ///
    /// Call it on the main thread once the event loop runs (macOS needs the
    /// application to exist).
    pub fn create(
        open: impl Fn() + Send + Sync + 'static,
        quit: impl Fn() + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        let menu = Menu::new();
        menu.append_items(&[
            &MenuItem::with_id(OPEN_ID, "Open cooldown.lol", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(QUIT_ID, "Quit cooldown.lol", true, None),
        ])?;

        let open = std::sync::Arc::new(open);
        let open_from_menu = open.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| match event.id.as_ref() {
            OPEN_ID => open_from_menu(),
            QUIT_ID => quit(),
            _ => {}
        }));
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
                && !platform::TRAY_MENU_ON_LEFT_CLICK
            {
                open();
            }
        }));

        let icon = TrayIconBuilder::new()
            .with_icon(crest()?)
            .with_tooltip("cooldown.lol")
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(platform::TRAY_MENU_ON_LEFT_CLICK)
            .build()?;
        Ok(Self { _icon: icon })
    }
}

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

/// The crest, the same image as the window icon (`docs/Design/Brand.md`).
const CREST_PNG: &[u8] = include_bytes!("../ui/images/app-icon.png");

/// Decodes the crest into the RGBA pixels the icon wants.
fn crest() -> anyhow::Result<Icon> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(CREST_PNG));
    decoder.set_transformations(png::Transformations::ALPHA | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("the crest is too large to decode"))?;
    let mut pixels = vec![0; size];
    let frame = reader.next_frame(&mut pixels)?;
    anyhow::ensure!(
        frame.color_type == png::ColorType::Rgba,
        "the crest must decode to RGBA, not {:?}",
        frame.color_type
    );
    pixels.truncate(frame.buffer_size());
    Ok(Icon::from_rgba(pixels, frame.width, frame.height)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crest_decodes() {
        assert!(crest().is_ok());
    }
}
