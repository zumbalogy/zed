use crate::{ItemHandle, Pane};
use gpui::{
    AnyView, App, Context, Decorations, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window,
};
use std::any::TypeId;
use theme::CLIENT_SIDE_DECORATION_ROUNDING;
use ui::{h_flex, prelude::*};
use util::ResultExt;

pub trait StatusItemView: Render {
    fn should_render(&self) -> bool;
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn crate::ItemHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    );
}

trait StatusItemViewHandle: Send {
    fn to_any(&self) -> Option<AnyView>;
    fn set_active_pane_item(
        &self,
        active_pane_item: Option<&dyn ItemHandle>,
        window: &mut Window,
        cx: &mut App,
    );
    fn item_type(&self) -> TypeId;
}

pub struct StatusBar {
    left_items: Vec<Box<dyn StatusItemViewHandle>>,
    right_items: Vec<Box<dyn StatusItemViewHandle>>,
    active_pane: Entity<Pane>,
    _observe_active_pane: Subscription,
}

impl Render for StatusBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // would anything be cleaner if this called .collect?
        let left_item_list = self.left_items.iter().filter_map(|x| x.to_any());
        let right_item_list = self.right_items.iter().rev().filter_map(|x| x.to_any());

        h_flex()
            .w_full()
            .justify_between()
            .gap(DynamicSpacing::Base08.rems(cx))
            .px(DynamicSpacing::Base06.rems(cx))
            .when(
                // Would like to avoid cloning
                left_item_list.clone().count() > 0 || right_item_list.clone().count() > 0,
                |this| this.py(DynamicSpacing::Base04.rems(cx)),
            )
            .bg(cx.theme().colors().status_bar_background)
            .map(|el| match window.window_decorations() {
                Decorations::Server => el,
                Decorations::Client { tiling, .. } => el
                    .when(!(tiling.bottom || tiling.right), |el| {
                        el.rounded_br(CLIENT_SIDE_DECORATION_ROUNDING)
                    })
                    .when(!(tiling.bottom || tiling.left), |el| {
                        el.rounded_bl(CLIENT_SIDE_DECORATION_ROUNDING)
                    })
                    // This border is to avoid a transparent gap in the rounded corners
                    .mb(px(-1.))
                    .border_b(px(1.0))
                    .border_color(cx.theme().colors().status_bar_background),
            })
            .child(self.render_children(left_item_list))
            .child(self.render_children(right_item_list))
    }
}

impl StatusBar {
    fn render_children(
        &self,
        items: impl IntoIterator<Item = impl IntoElement>,
    ) -> impl IntoElement {
        h_flex().gap_1().overflow_x_hidden().children(items)
    }
}

impl StatusBar {
    pub fn new(active_pane: &Entity<Pane>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            left_items: Default::default(),
            right_items: Default::default(),
            active_pane: active_pane.clone(),
            _observe_active_pane: cx.observe_in(active_pane, window, |this, _, window, cx| {
                this.update_active_pane_item(window, cx)
            }),
        };
        this.update_active_pane_item(window, cx);
        this
    }

    pub fn add_left_item<T>(&mut self, item: Entity<T>, window: &mut Window, cx: &mut Context<Self>)
    where
        T: 'static + StatusItemView,
    {
        let active_pane_item = self.active_pane.read(cx).active_item();
        item.set_active_pane_item(active_pane_item.as_deref(), window, cx);

        self.left_items.push(Box::new(item));
        cx.notify();
    }

    pub fn item_of_type<T: StatusItemView>(&self) -> Option<Entity<T>> {
        self.left_items
            .iter()
            .chain(self.right_items.iter())
            .find_map(|item| item.to_any()?.downcast().log_err())
    }

    pub fn position_of_item<T>(&self) -> Option<usize>
    where
        T: StatusItemView,
    {
        for (index, item) in self.left_items.iter().enumerate() {
            if item.item_type() == TypeId::of::<T>() {
                return Some(index);
            }
        }
        for (index, item) in self.right_items.iter().enumerate() {
            if item.item_type() == TypeId::of::<T>() {
                return Some(index + self.left_items.len());
            }
        }
        None
    }

    pub fn insert_item_after<T>(
        &mut self,
        position: usize,
        item: Entity<T>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        T: 'static + StatusItemView,
    {
        let active_pane_item = self.active_pane.read(cx).active_item();
        item.set_active_pane_item(active_pane_item.as_deref(), window, cx);

        if position < self.left_items.len() {
            self.left_items.insert(position + 1, Box::new(item))
        } else {
            self.right_items
                .insert(position + 1 - self.left_items.len(), Box::new(item))
        }
        cx.notify()
    }

    pub fn remove_item_at(&mut self, position: usize, cx: &mut Context<Self>) {
        if position < self.left_items.len() {
            self.left_items.remove(position);
        } else {
            self.right_items.remove(position - self.left_items.len());
        }
        cx.notify();
    }

    pub fn add_right_item<T>(
        &mut self,
        item: Entity<T>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        T: 'static + StatusItemView,
    {
        let active_pane_item = self.active_pane.read(cx).active_item();
        item.set_active_pane_item(active_pane_item.as_deref(), window, cx);

        self.right_items.push(Box::new(item));
        cx.notify();
    }

    pub fn set_active_pane(
        &mut self,
        active_pane: &Entity<Pane>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_pane = active_pane.clone();
        self._observe_active_pane = cx.observe_in(active_pane, window, |this, _, window, cx| {
            this.update_active_pane_item(window, cx)
        });
        self.update_active_pane_item(window, cx);
    }

    fn update_active_pane_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_pane_item = self.active_pane.read(cx).active_item();
        for item in self.left_items.iter().chain(&self.right_items) {
            item.set_active_pane_item(active_pane_item.as_deref(), window, cx);
        }
    }
}

impl<T: StatusItemView> StatusItemViewHandle for Entity<T> {
    fn to_any(&self) -> Option<AnyView> {
        // if self.should_render() {
        if true {
            Some(self.clone().into())
        } else {
            None
        }
    }

    fn set_active_pane_item(
        &self,
        active_pane_item: Option<&dyn ItemHandle>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.update(cx, |this, cx| {
            this.set_active_pane_item(active_pane_item, window, cx)
        });
    }

    fn item_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl From<&dyn StatusItemViewHandle> for Option<AnyView> {
    fn from(val: &dyn StatusItemViewHandle) -> Option<AnyView> {
        val.to_any()
    }
}
