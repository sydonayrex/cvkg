use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// Navigation stack for push/pop navigation
pub struct NavigationStack {
    pub(crate) stack: Vec<cvkg_core::AnyView>,
}

impl NavigationStack {
    pub fn new<V: View + 'static>(root: V) -> Self {
        Self { stack: vec![root.erase()] }
    }

    pub fn push<V: View + 'static>(&mut self, view: V) {
        self.stack.push(view.erase());
    }

    pub fn pop(&mut self) -> Option<cvkg_core::AnyView> {
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }
}

impl View for NavigationStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NavigationStack");
        // Render the top-most view in the stack
        if let Some(top) = self.stack.last() {
            top.render(renderer, rect);
        }
        renderer.pop_vnode();
    }
}

/// Navigation split view for sidebar layouts
pub struct NavigationSplitView<S, D> {
    pub(crate) sidebar: S,
    pub(crate) detail: D,
}

impl<S: View, D: View> NavigationSplitView<S, D> {
    pub fn new(sidebar: S, detail: D) -> Self {
        Self { sidebar, detail }
    }
}

impl<S: View, D: View> View for NavigationSplitView<S, D> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let sidebar_width = (rect.width * 0.3).min(300.0);
        let sidebar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: sidebar_width,
            height: rect.height,
        };
        let detail_rect = Rect {
            x: rect.x + sidebar_width,
            y: rect.y,
            width: rect.width - sidebar_width,
            height: rect.height,
        };

        // Render sidebar with a subtle background
        renderer.fill_rect(sidebar_rect, [0.05, 0.05, 0.08, 1.0]);
        renderer.stroke_rect(sidebar_rect, [0.2, 0.2, 0.3, 0.5], 1.0);
        self.sidebar.render(renderer, sidebar_rect);

        // Render detail area
        self.detail.render(renderer, detail_rect);
    }
}

/// Tab bar navigation view
pub struct TabView<V> {
    pub(crate) content: V,
}

impl<V: View> TabView<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for TabView<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let tab_bar_height = 50.0;
        let content_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height - tab_bar_height,
        };
        let tab_bar_rect = Rect {
            x: rect.x,
            y: rect.y + rect.height - tab_bar_height,
            width: rect.width,
            height: tab_bar_height,
        };

        // Render content
        self.content.render(renderer, content_rect);

        // Render tab bar background
        renderer.bifrost(tab_bar_rect, 10.0, 1.2, 0.9);
        renderer.fill_rect(tab_bar_rect, [0.0, 0.0, 0.0, 0.5]);
        renderer.draw_line(tab_bar_rect.x, tab_bar_rect.y, tab_bar_rect.x + tab_bar_rect.width, tab_bar_rect.y, [0.3, 0.3, 0.4, 1.0], 1.0);
    }
}

/// Modal bottom sheet or centered dialog
pub struct Sheet<V> {
    pub(crate) content: V,
    pub(crate) is_presented: bool,
}

impl<V: View> Sheet<V> {
    pub fn new(content: V, is_presented: bool) -> Self {
        Self {
            content,
            is_presented,
        }
    }
}

impl<V: View> View for Sheet<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_presented {
            return;
        }

        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.7]);

        let modal_width = (rect.width * 0.8).min(500.0);
        let modal_height = (rect.height * 0.6).min(400.0);
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_width) / 2.0,
            y: rect.y + (rect.height - modal_height) / 2.0,
            width: modal_width,
            height: modal_height,
        };

        renderer.bifrost(modal_rect, 25.0, 1.5, 0.85);
        renderer.fill_rounded_rect(modal_rect, 12.0, [0.0, 0.0, 0.0, 0.3]);
        renderer.stroke_rounded_rect(modal_rect, 12.0, [0.2, 0.25, 0.3, 0.6], 2.0);

        self.content.render(renderer, modal_rect);
    }
}

/// A modifier that presents a modal sheet over a view.
#[derive(Clone)]
pub struct SheetModifier<V2> {
    pub is_presented: bool,
    pub content: V2,
}

impl<V2: View + Clone> cvkg_core::ViewModifier for SheetModifier<V2> {
    fn modify<V: View>(self, content: V) -> impl View {
        cvkg_core::ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        view.render(renderer, rect);

        if self.is_presented {
            renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.7]);

            let modal_width = (rect.width * 0.8).min(500.0);
            let modal_height = (rect.height * 0.6).min(400.0);
            let modal_rect = Rect {
                x: rect.x + (rect.width - modal_width) / 2.0,
                y: rect.y + (rect.height - modal_height) / 2.0,
                width: modal_width,
                height: modal_height,
            };

            renderer.bifrost(modal_rect, 25.0, 1.5, 0.85);
            renderer.fill_rounded_rect(modal_rect, 12.0, [0.0, 0.0, 0.0, 0.3]);
            renderer.stroke_rounded_rect(modal_rect, 12.0, [0.2, 0.25, 0.3, 0.6], 2.0);

            self.content.render(renderer, modal_rect);
        }
    }
}

/// A modal dialog with title, content, and actions.
pub struct Dialog<V> {
    pub(crate) is_presented: bool,
    pub(crate) title: Option<String>,
    pub(crate) content: V,
    pub(crate) actions: Vec<DialogAction>,
}

impl<V: View> Dialog<V> {
    pub fn new(content: V) -> Self {
        Self {
            is_presented: false,
            title: None,
            content,
            actions: Vec::new(),
        }
    }

    pub fn presented(mut self, is_presented: bool) -> Self {
        self.is_presented = is_presented;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn action(
        mut self,
        label: impl Into<String>,
        on_click: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.actions.push(DialogAction {
            label: label.into(),
            style: DialogActionStyle::Default,
            on_click: std::sync::Arc::new(on_click),
        });
        self
    }
}

impl<V: View> View for Dialog<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_presented {
            return;
        }

        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.7]);

        let modal_w = (rect.width * 0.8).min(450.0);
        let modal_h = (rect.height * 0.5).min(350.0);
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_w) / 2.0,
            y: rect.y + (rect.height - modal_h) / 2.0,
            width: modal_w,
            height: modal_h,
        };

        renderer.bifrost(modal_rect, 20.0, 1.2, 0.9);
        renderer.fill_rounded_rect(modal_rect, 12.0, [0.05, 0.05, 0.1, 0.8]);
        renderer.stroke_rounded_rect(modal_rect, 12.0, [0.0, 0.8, 1.0, 0.5], 1.5);

        let padding = 20.0;
        let mut current_y = modal_rect.y + padding;

        if let Some(title) = &self.title {
            renderer.draw_text(
                title,
                modal_rect.x + padding,
                current_y,
                20.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            current_y += 30.0;
        }

        let content_h = modal_h - (current_y - modal_rect.y) - 60.0;
        let content_rect = Rect {
            x: modal_rect.x + padding,
            y: current_y,
            width: modal_w - 2.0 * padding,
            height: content_h,
        };
        self.content.render(renderer, content_rect);

        let action_y = modal_rect.y + modal_h - 45.0;
        let action_w = 80.0;
        for (i, action) in self.actions.iter().enumerate() {
            let action_rect = Rect {
                x: modal_rect.x + modal_w - padding - (i as f32 + 1.0) * (action_w + 10.0),
                y: action_y,
                width: action_w,
                height: 30.0,
            };
            renderer.fill_rounded_rect(action_rect, 4.0, [0.15, 0.15, 0.2, 1.0]);
            renderer.stroke_rect(action_rect, [0.0, 0.8, 1.0, 0.8], 1.0);
            renderer.draw_text(
                &action.label,
                action_rect.x + 8.0,
                action_rect.y + 8.0,
                14.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }
}

pub struct DialogAction {
    pub label: String,
    pub style: DialogActionStyle,
    pub on_click: std::sync::Arc<dyn Fn() + Send + Sync>,
}

pub enum DialogActionStyle {
    Default,
    Destructive,
    Cancel,
}

pub struct AlertDialog {
    pub(crate) is_presented: bool,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) on_confirm: std::sync::Arc<dyn Fn() + Send + Sync>,
    pub(crate) on_cancel: std::sync::Arc<dyn Fn() + Send + Sync>,
}

impl AlertDialog {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            is_presented: false,
            title: title.into(),
            description: description.into(),
            on_confirm: std::sync::Arc::new(|| {}),
            on_cancel: std::sync::Arc::new(|| {}),
        }
    }

    pub fn presented(mut self, is_presented: bool) -> Self {
        self.is_presented = is_presented;
        self
    }

    pub fn on_confirm(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_confirm = std::sync::Arc::new(callback);
        self
    }

    pub fn on_cancel(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = std::sync::Arc::new(callback);
        self
    }
}

impl View for AlertDialog {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_presented {
            return;
        }

        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.7]);
        let modal_w = 400.0;
        let modal_h = 200.0;
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_w) / 2.0,
            y: rect.y + (rect.height - modal_h) / 2.0,
            width: modal_w,
            height: modal_h,
        };

        renderer.bifrost(modal_rect, 20.0, 1.2, 0.9);
        renderer.fill_rounded_rect(modal_rect, 12.0, [0.08, 0.08, 0.1, 0.9]);
        renderer.stroke_rounded_rect(modal_rect, 12.0, [1.0, 0.2, 0.2, 0.6], 2.0);

        renderer.draw_text(
            &self.title,
            modal_rect.x + 20.0,
            modal_rect.y + 20.0,
            22.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.draw_text(
            &self.description,
            modal_rect.x + 20.0,
            modal_rect.y + 55.0,
            14.0,
            [0.8, 0.8, 0.8, 1.0],
        );

        let btn_w = 100.0;
        let btn_h = 36.0;
        let cancel_rect = Rect {
            x: modal_rect.x + modal_w - 230.0,
            y: modal_rect.y + modal_h - 56.0,
            width: btn_w,
            height: btn_h,
        };
        let confirm_rect = Rect {
            x: modal_rect.x + modal_w - 120.0,
            y: modal_rect.y + modal_h - 56.0,
            width: btn_w,
            height: btn_h,
        };

        renderer.fill_rounded_rect(cancel_rect, 6.0, [0.2, 0.2, 0.25, 1.0]);
        renderer.draw_text(
            "Cancel",
            cancel_rect.x + 25.0,
            cancel_rect.y + 10.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        renderer.fill_rounded_rect(confirm_rect, 6.0, [0.8, 0.1, 0.1, 1.0]);
        renderer.draw_text(
            "Confirm",
            confirm_rect.x + 20.0,
            confirm_rect.y + 10.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

/// Context menu dropdown
pub struct Menu<V> {
    pub(crate) content: V,
}

impl<V: View> Menu<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Menu<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render menu as a floating glass box
        renderer.bifrost(rect, 15.0, 1.1, 0.95);
        renderer.fill_rounded_rect(rect, 8.0, [0.1, 0.1, 0.15, 0.9]);
        renderer.stroke_rounded_rect(rect, 8.0, [0.0, 0.8, 1.0, 0.4], 1.0);
        self.content.render(renderer, rect);
    }
}

/// Scrollable container for content that exceeds available space
pub struct ScrollView<V> {
    pub(crate) content: V,
    pub(crate) scroll_offset: f32,
}

impl<V: View> ScrollView<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            scroll_offset: 0.0,
        }
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.scroll_offset = offset;
        self
    }
}

impl<V: View> View for ScrollView<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_clip_rect(rect);
        let content_rect = Rect {
            x: rect.x,
            y: rect.y - self.scroll_offset,
            width: rect.width,
            height: rect.height,
        };
        self.content.render(renderer, content_rect);
        renderer.pop_clip_rect();
    }
}

/// Multi-column table layout
pub struct Table<V> {
    pub(crate) content: V,
}

impl<V: View> Table<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Table<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Table layout logic would go here
        self.content.render(renderer, rect);
    }
}

/// Settings style grouped form layout
pub struct Form<V> {
    pub(crate) content: V,
}

impl<V: View> Form<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Form<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Grouped form layout logic
        self.content.render(renderer, rect);
    }
}

/// A vertical stack of views
pub struct VStack {
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    children: Vec<cvkg_core::AnyView>,
}

impl VStack {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            alignment: cvkg_core::Alignment::Center,
            distribution: cvkg_core::Distribution::Fill,
            children: Vec::new(),
        }
    }

    pub fn alignment(mut self, alignment: cvkg_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn distribution(mut self, distribution: cvkg_core::Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    pub fn child<V: View + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for VStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "VStack");
        if self.children.is_empty() {
            renderer.pop_vnode();
            return;
        }

        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let rects = cvkg_layout::VStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            rect,
            &layouts,
            &mut cache,
        );

        let mut rect_idx = 0;
        for child in self.children.iter() {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            width = width.max(child_size.width);
            height += child_size.height;
            if i < self.children.len() - 1 {
                height += self.spacing;
            }
        }

        Size { width, height }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for VStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            if let Some(layout) = child.layout() {
                let child_size = layout.size_that_fits(proposal, &[], cache);
                width = width.max(child_size.width);
                height += child_size.height;
                if i < self.children.len() - 1 {
                    height += self.spacing;
                }
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let _rects = cvkg_layout::VStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &layouts,
            cache,
        );
    }
}

/// A vertical stack that only renders visible children (efficient for long lists)
pub struct LazyVStack {
    spacing: f32,
    children: Vec<cvkg_core::AnyView>,
}

impl LazyVStack {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            children: Vec::new(),
        }
    }

    pub fn child<V: View + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for LazyVStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let child_height = 40.0;

        for (i, child) in self.children.iter().enumerate() {
            let child_y = rect.y + i as f32 * (child_height + self.spacing);

            if child_y + child_height < 0.0 {
                continue;
            }
            if child_y > 2000.0 {
                break;
            }

            child.render(
                renderer,
                Rect {
                    x: rect.x,
                    y: child_y,
                    width: rect.width,
                    height: child_height,
                },
            );
        }
    }
}

/// A horizontal stack of views
pub struct HStack {
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    children: Vec<cvkg_core::AnyView>,
}

impl HStack {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            alignment: cvkg_core::Alignment::Center,
            distribution: cvkg_core::Distribution::Fill,
            children: Vec::new(),
        }
    }

    pub fn alignment(mut self, alignment: cvkg_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn distribution(mut self, distribution: cvkg_core::Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    pub fn child<V: View + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for HStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let rects = cvkg_layout::HStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            rect,
            &layouts,
            &mut cache,
        );

        let mut rect_idx = 0;
        for child in self.children.iter() {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            width += child_size.width;
            height = height.max(child_size.height);
            if i < self.children.len() - 1 {
                width += self.spacing;
            }
        }

        Size { width, height }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for HStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            if let Some(layout) = child.layout() {
                let child_size = layout.size_that_fits(proposal, &[], cache);
                width += child_size.width;
                height = height.max(child_size.height);
                if i < self.children.len() - 1 {
                    width += self.spacing;
                }
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let _rects = cvkg_layout::HStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &layouts,
            cache,
        );
    }
}

/// A flexible container that defaults to a glassmorphic construct over a void black background
pub struct FlexBox {
    pub orientation: cvkg_core::Orientation,
    pub spacing: f32,
    children: Vec<cvkg_core::AnyView>,
}

impl FlexBox {
    pub fn new(orientation: cvkg_core::Orientation, spacing: f32) -> Self {
        Self {
            orientation,
            spacing,
            children: Vec::new(),
        }
    }

    pub fn child<V: View + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for FlexBox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 8.0, [0.0, 0.0, 0.0, 0.85]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.25, 0.5], 1.0);
        renderer.bifrost(rect, 15.0, 1.2, 0.85);

        if self.children.is_empty() {
            return;
        }

        let n = self.children.len() as f32;
        match self.orientation {
            cvkg_core::Orientation::Horizontal => {
                let total_spacing = self.spacing * (n - 1.0);
                let item_width = (rect.width - total_spacing) / n;
                for (i, child) in self.children.iter().enumerate() {
                    let child_rect = Rect {
                        x: rect.x + i as f32 * (item_width + self.spacing),
                        y: rect.y,
                        width: item_width,
                        height: rect.height,
                    };
                    child.render(renderer, child_rect);
                }
            }
            cvkg_core::Orientation::Vertical => {
                let total_spacing = self.spacing * (n - 1.0);
                let item_height = (rect.height - total_spacing) / n;
                for (i, child) in self.children.iter().enumerate() {
                    let child_rect = Rect {
                        x: rect.x,
                        y: rect.y + i as f32 * (item_height + self.spacing),
                        width: rect.width,
                        height: item_height,
                    };
                    child.render(renderer, child_rect);
                }
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            match self.orientation {
                cvkg_core::Orientation::Horizontal => {
                    width += child_size.width;
                    height = height.max(child_size.height);
                    if i < self.children.len() - 1 {
                        width += self.spacing;
                    }
                }
                cvkg_core::Orientation::Vertical => {
                    width = width.max(child_size.width);
                    height += child_size.height;
                    if i < self.children.len() - 1 {
                        height += self.spacing;
                    }
                }
            }
        }

        Size { width, height }
    }
}

/// Tooltip component for displaying short messages on hover.
pub struct Tooltip<V> {
    pub(crate) content: V,
    pub(crate) text: String,
    pub(crate) position: TooltipPosition,
}

impl<V: View> Tooltip<V> {
    pub fn new(content: V, text: impl Into<String>) -> Self {
        Self {
            content,
            text: text.into(),
            position: TooltipPosition::Top,
        }
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }
}

pub enum TooltipPosition {
    Top,
    Right,
    Bottom,
    Left,
}

impl<V: View> View for Tooltip<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.content.render(renderer, rect);

        let (tw, th) = renderer.measure_text(&self.text, 12.0);
        let bubble_w = tw + 16.0;
        let bubble_h = th + 8.0;

        let bubble_rect = match self.position {
            TooltipPosition::Top => Rect { x: rect.x + (rect.width - bubble_w) / 2.0, y: rect.y - bubble_h - 5.0, width: bubble_w, height: bubble_h },
            TooltipPosition::Bottom => Rect { x: rect.x + (rect.width - bubble_w) / 2.0, y: rect.y + rect.height + 5.0, width: bubble_w, height: bubble_h },
            TooltipPosition::Left => Rect { x: rect.x - bubble_w - 5.0, y: rect.y + (rect.height - bubble_h) / 2.0, width: bubble_w, height: bubble_h },
            TooltipPosition::Right => Rect { x: rect.x + rect.width + 5.0, y: rect.y + (rect.height - bubble_h) / 2.0, width: bubble_w, height: bubble_h },
        };

        renderer.fill_rounded_rect(bubble_rect, 4.0, [0.05, 0.05, 0.1, 0.9]);
        renderer.stroke_rounded_rect(bubble_rect, 4.0, [0.0, 0.8, 1.0, 0.5], 1.0);
        renderer.draw_text(&self.text, bubble_rect.x + 8.0, bubble_rect.y + 4.0, 12.0, [1.0, 1.0, 1.0, 1.0]);
    }
}

/// Popover component for displaying rich content in a floating bubble.
pub struct Popover<T, C> {
    pub(crate) trigger: T,
    pub(crate) content: C,
    pub(crate) is_open: bool,
    pub(crate) position: PopoverPosition,
}

impl<T: View, C: View> Popover<T, C> {
    pub fn new(trigger: T, content: C) -> Self {
        Self {
            trigger,
            content,
            is_open: false,
            position: PopoverPosition::Bottom,
        }
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }
}

pub enum PopoverPosition {
    Top,
    Right,
    Bottom,
    Left,
}

impl<T: View, C: View> View for Popover<T, C> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.trigger.render(renderer, rect);

        if self.is_open {
            let popover_w = 200.0;
            let popover_h = 150.0;

            let popover_rect = match self.position {
                PopoverPosition::Bottom => Rect { x: rect.x + (rect.width - popover_w) / 2.0, y: rect.y + rect.height + 8.0, width: popover_w, height: popover_h },
                _ => Rect { x: rect.x, y: rect.y + rect.height + 8.0, width: popover_w, height: popover_h },
            };

            renderer.bifrost(popover_rect, 15.0, 1.2, 0.9);
            renderer.fill_rounded_rect(popover_rect, 8.0, [0.05, 0.05, 0.1, 0.95]);
            renderer.stroke_rounded_rect(popover_rect, 8.0, [0.0, 1.0, 1.0, 0.4], 1.5);
            self.content.render(renderer, popover_rect);
        }
    }
}

/// Accordion component for collapsible content sections.
pub struct Accordion<V> {
    pub(crate) items: Vec<AccordionItem<V>>,
}

impl<V: View> Accordion<V> {
    pub fn new() -> Self { Self { items: Vec::new() } }
    pub fn item(mut self, title: impl Into<String>, content: V) -> Self {
        self.items.push(AccordionItem { title: title.into(), content, is_expanded: false });
        self
    }
}

pub struct AccordionItem<V> {
    pub(crate) title: String,
    pub(crate) content: V,
    pub(crate) is_expanded: bool,
}

impl<V: View> View for Accordion<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut current_y = rect.y;
        for item in &self.items {
            let header_h = 32.0;
            let header_rect = Rect { x: rect.x, y: current_y, width: rect.width, height: header_h };
            
            renderer.fill_rounded_rect(header_rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
            renderer.draw_text(&item.title, header_rect.x + 8.0, header_rect.y + 8.0, 14.0, [1.0, 1.0, 1.0, 1.0]);
            renderer.draw_text(if item.is_expanded { "▼" } else { "▶" }, header_rect.x + rect.width - 20.0, header_rect.y + 8.0, 12.0, [0.6, 0.6, 0.7, 1.0]);
            
            current_y += header_h + 4.0;
            if item.is_expanded {
                let content_h = 100.0; // Simplified height
                let content_rect = Rect { x: rect.x + 8.0, y: current_y, width: rect.width - 16.0, height: content_h };
                item.content.render(renderer, content_rect);
                current_y += content_h + 8.0;
            }
        }
    }
}

/// Collapsible component for hiding/showing content.
pub struct Collapsible<V> {
    pub(crate) content: V,
    pub(crate) is_open: bool,
}

impl<V: View> Collapsible<V> {
    pub fn new(content: V, is_open: bool) -> Self {
        Self { content, is_open }
    }
}

impl<V: View> View for Collapsible<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.is_open {
            self.content.render(renderer, rect);
        }
    }
}
