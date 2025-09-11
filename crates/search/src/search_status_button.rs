use editor::EditorSettings;
use gpui::Empty;
use settings::Settings as _;
use ui::{ButtonCommon, Clickable, Context, Render, Tooltip, Window, prelude::*};

use workspace::{ItemHandle, StatusItemView};

pub const SEARCH_ICON: IconName = IconName::MagnifyingGlass;

pub struct SearchButton;

impl SearchButton {
    pub fn new() -> Self {
        Self {}
    }
}

impl Render for SearchButton {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !EditorSettings::get_global(cx).search.button {
            return Empty.into_any_element();
        }

        div()
            .child(
                IconButton::new("project-search-indicator", SEARCH_ICON)
                    .icon_size(IconSize::Small)
                    .tooltip(|window, cx| {
                        Tooltip::for_action(
                            "Project Search",
                            &workspace::DeploySearch::default(),
                            window,
                            cx,
                        )
                    })
                    .on_click(cx.listener(|_this, _, window, cx| {
                        window.dispatch_action(Box::new(workspace::DeploySearch::default()), cx);
                    })),
            )
            .into_any_element()
    }
}

impl StatusItemView for SearchButton {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }
}
