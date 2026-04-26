use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// Navigation stack for push/pop navigation
#[allow(dead_code)]
pub struct NavigationStack<V> {
    pub(crate) root: V,
}

impl<V: View> NavigationStack<V> {
    /// Create a new NavigationStack.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{NavigationStack, Text};
    /// let nav = NavigationStack::new(Text::new("Root"));
    /// ```
    pub fn new(root: V) -> Self {
        Self { root }
    }
}

impl<V: View> View for NavigationStack<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Navigation split view for sidebar layouts
#[allow(dead_code)]
pub struct NavigationSplitView<S, D> {
    pub(crate) sidebar: S,
    pub(crate) detail: D,
}

impl<S: View, D: View> NavigationSplitView<S, D> {
    /// Create a new NavigationSplitView.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{NavigationSplitView, Text};
    /// let split = NavigationSplitView::new(Text::new("Sidebar"), Text::new("Detail"));
    /// ```
    pub fn new(sidebar: S, detail: D) -> Self {
        Self { sidebar, detail }
    }
}

impl<S: View, D: View> View for NavigationSplitView<S, D> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Tab bar navigation view
#[allow(dead_code)]
pub struct TabView<V> {
    pub(crate) content: V,
}

impl<V: View> TabView<V> {
    /// Create a new TabView.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{TabView, Text};
    /// let tabs = TabView::new(Text::new("Tabs"));
    /// ```
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for TabView<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Modal bottom sheet or centered dialog
#[allow(dead_code)]
pub struct Sheet<V> {
    pub(crate) content: V,
    pub(crate) is_presented: bool,
}

impl<V: View> Sheet<V> {
    /// Create a new Sheet.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{Sheet, Text};
    /// let sheet = Sheet::new(Text::new("Content"), true);
    /// ```
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

        // Render dimming background (Ginnungagap Void)
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.7]);

        let modal_width = (rect.width * 0.8).min(500.0);
        let modal_height = (rect.height * 0.6).min(400.0);
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_width) / 2.0,
            y: rect.y + (rect.height - modal_height) / 2.0,
            width: modal_width,
            height: modal_height,
        };

        // Render frosted glass background for the modal edges (evident frosting)
        renderer.bifrost(modal_rect, 25.0, 1.5, 0.85);
        // Mostly clear center
        renderer.fill_rounded_rect(modal_rect, 12.0, [0.0, 0.0, 0.0, 0.3]);
        // Neon glass border
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
        // Render the underlying view first
        view.render(renderer, rect);

        // If presented, render the modal on top
        if self.is_presented {
            // Render dimming background (Ginnungagap Void)
            renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.7]);

            let modal_width = (rect.width * 0.8).min(500.0);
            let modal_height = (rect.height * 0.6).min(400.0);
            let modal_rect = Rect {
                x: rect.x + (rect.width - modal_width) / 2.0,
                y: rect.y + (rect.height - modal_height) / 2.0,
                width: modal_width,
                height: modal_height,
            };

            // Evident frosting of the glass around the edges
            renderer.bifrost(modal_rect, 25.0, 1.5, 0.85);
            // Mostly clear center
            renderer.fill_rounded_rect(modal_rect, 12.0, [0.0, 0.0, 0.0, 0.3]);
            // Neon glass border
            renderer.stroke_rounded_rect(modal_rect, 12.0, [0.2, 0.25, 0.3, 0.6], 2.0);

            self.content.render(renderer, modal_rect);
        }
    }
}

/// System alert dialog
#[allow(dead_code)]
pub struct Alert {
    pub(crate) title: String,
    pub(crate) is_presented: bool,
}

impl Alert {
    /// Create a new Alert.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Alert;
    /// let alert = Alert::new("Warning", true);
    /// ```
    pub fn new(title: impl Into<String>, is_presented: bool) -> Self {
        Self {
            title: title.into(),
            is_presented,
        }
    }
}

impl View for Alert {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Action sheet confirmation dialog
#[allow(dead_code)]
pub struct ConfirmationDialog {
    pub(crate) title: String,
    pub(crate) is_presented: bool,
}

impl ConfirmationDialog {
    /// Create a new ConfirmationDialog.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::ConfirmationDialog;
    /// let dialog = ConfirmationDialog::new("Are you sure?", true);
    /// ```
    pub fn new(title: impl Into<String>, is_presented: bool) -> Self {
        Self {
            title: title.into(),
            is_presented,
        }
    }
}

impl View for ConfirmationDialog {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Context menu dropdown
#[allow(dead_code)]
pub struct Menu<V> {
    pub(crate) content: V,
}

impl<V: View> Menu<V> {
    /// Create a new Menu.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{Menu, Text};
    /// let menu = Menu::new(Text::new("Options"));
    /// ```
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Menu<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Scrollable container for content that exceeds available space
#[allow(dead_code)]
pub struct ScrollView<V> {
    pub(crate) content: V,
    pub(crate) scroll_offset: f32,
}

impl<V: View> ScrollView<V> {
    /// Create a new ScrollView.
    pub fn new(content: V) -> Self {
        Self {
            content,
            scroll_offset: 0.0,
        }
    }

    /// Set the scroll offset.
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
        // Clip to the ScrollView's visible bounds before rendering offset content.
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
#[allow(dead_code)]
pub struct Table<V> {
    pub(crate) content: V,
}

impl<V: View> Table<V> {
    /// Create a new Table.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{Table, Text};
    /// let table = Table::new(Text::new("Row 1"));
    /// ```
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Table<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

/// Settings style grouped form layout
#[allow(dead_code)]
pub struct Form<V> {
    pub(crate) content: V,
}

impl<V: View> Form<V> {
    /// Create a new Form.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{Form, Text};
    /// let form = Form::new(Text::new("Field 1"));
    /// ```
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Form<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
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
        if self.children.is_empty() {
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

        for (i, child) in self.children.iter().enumerate() {
            if i < rects.len() {
                child.render(renderer, rects[i]);
            }
        }
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
        // Note: in a full recursive layout engine, we would call place_subviews on children here.
    }
}

/// A vertical stack that only renders visible children (efficient for long lists)
pub struct LazyVStack {
    spacing: f32,
    children: Vec<cvkg_core::AnyView>,
}

impl LazyVStack {
    /// Create a new LazyVStack.
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            children: Vec::new(),
        }
    }

    /// Add a child to the stack.
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

            // Basic visibility check (Lazy Rendering)
            if child_y + child_height < 0.0 {
                continue;
            }
            if child_y > 2000.0 {
                break;
            } // Assuming a reasonable viewport height

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

        for (i, child) in self.children.iter().enumerate() {
            if i < rects.len() {
                child.render(renderer, rects[i]);
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
        // Natural void black background with glassmorphic construct
        renderer.fill_rounded_rect(rect, 8.0, [0.0, 0.0, 0.0, 0.85]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.25, 0.5], 1.0);
        renderer.bifrost(rect, 15.0, 1.2, 0.85); // Glassmorphism

        if self.children.is_empty() {
            return;
        }

        // Use the flex layout to compute placements.
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
