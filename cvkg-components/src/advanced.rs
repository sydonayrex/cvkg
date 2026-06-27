use crate::theme;
use crate::{RADIUS_LG, RADIUS_MD};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

// Overlay & Disclosure
#[derive(Clone, Default)]
pub struct FafnirAccordion<H, C> {
    items: Vec<(H, C)>,
    open_index: Option<usize>,
}

impl<H, C> FafnirAccordion<H, C> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            open_index: None,
        }
    }

    pub fn item(mut self, header: H, content: C) -> Self {
        self.items.push((header, content));
        self
    }

    pub fn open_index(mut self, index: usize) -> Self {
        self.open_index = Some(index);
        self
    }
}

impl<H: View, C: View> View for FafnirAccordion<H, C> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FafnirAccordion");

        let mut y_offset = 0.0;
        let spacing = 8.0;

        for (i, (header, content)) in self.items.iter().enumerate() {
            let is_open = self.open_index == Some(i);

            // Render Header
            let header_size = header.intrinsic_size(renderer, SizeProposal::width(rect.width));
            let header_rect = Rect::new(rect.x, rect.y + y_offset, rect.width, header_size.height);

            // Header background
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(header_rect, 8.0, 1.0, 0.8);
            }
            renderer.stroke_rounded_rect(header_rect, RADIUS_LG, theme::border(), 1.0);

            header.render(renderer, header_rect.inset(12.0)); // 12px padding
            y_offset += header_size.height + spacing;

            // Render Content if open
            if is_open {
                let content_size =
                    content.intrinsic_size(renderer, SizeProposal::width(rect.width));
                let content_rect =
                    Rect::new(rect.x, rect.y + y_offset, rect.width, content_size.height);

                // Content background
                renderer.fill_rounded_rect(
                    content_rect,
                    RADIUS_LG,
                    theme::with_alpha(theme::surface_elevated(), 0.6),
                );

                content.render(renderer, content_rect.inset(12.0));
                y_offset += content_size.height + spacing;
            }
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut total_height = 0.0;
        let mut max_width = 0.0_f32;
        let spacing = 8.0;

        for (i, (header, content)) in self.items.iter().enumerate() {
            let header_size = header.intrinsic_size(renderer, proposal);
            total_height += header_size.height + spacing;
            max_width = max_width.max(header_size.width);

            if self.open_index == Some(i) {
                let content_size = content.intrinsic_size(renderer, proposal);
                total_height += content_size.height + spacing;
                max_width = max_width.max(content_size.width);
            }
        }

        Size {
            width: max_width.max(proposal.width.unwrap_or(200.0)),
            height: total_height,
        }
    }
}

#[derive(Clone)]
pub struct SvadilVeil<V> {
    content: Option<V>,
    is_active: bool,
}
impl<V> SvadilVeil<V> {
    pub fn new() -> Self {
        Self {
            content: None,
            is_active: false,
        }
    }
    pub fn content(mut self, c: V) -> Self {
        self.content = Some(c);
        self
    }
    pub fn active(mut self, a: bool) -> Self {
        self.is_active = a;
        self
    }
}
impl<V> Default for SvadilVeil<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for SvadilVeil<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_active {
            return;
        }
        renderer.push_vnode(rect, "SvadilVeil");
        // Apply full screen blur / frosted glass background
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(rect, 0.0, 10.0, 0.6);
        }
        renderer.fill_rect(rect, theme::bg()); // Dark dim

        if let Some(content) = &self.content {
            let size =
                content.intrinsic_size(renderer, SizeProposal::tight(rect.width, rect.height));
            // Delegate centering to the layout engine via HStack+VStack
            let inner = Rect::new(rect.x, rect.y, rect.width, rect.height);
            let mut cache = LayoutCache::new();
            let layouts: Vec<&dyn LayoutView> = vec![];
            let row = cvkg_layout::HStack::compute_layout(
                0.0,
                cvkg_core::Alignment::Center,
                cvkg_core::Distribution::Center,
                inner,
                &layouts,
                &mut cache,
            );
            let cx = if !row.is_empty() { row[0].x } else { rect.x };
            let cy = if !row.is_empty() { row[0].y } else { rect.y };
            content.render(renderer, Rect::new(cx, cy, size.width, size.height));
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(100.0),
            height: proposal.height.unwrap_or(100.0),
        } // Fills available space
    }
}

#[derive(Clone)]
pub struct HuginHoverCard<V> {
    content: Option<V>,
    is_active: bool,
}
impl<V> HuginHoverCard<V> {
    pub fn new() -> Self {
        Self {
            content: None,
            is_active: false,
        }
    }
    pub fn content(mut self, c: V) -> Self {
        self.content = Some(c);
        self
    }
    pub fn active(mut self, a: bool) -> Self {
        self.is_active = a;
        self
    }
}
impl<V> Default for HuginHoverCard<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for HuginHoverCard<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_active {
            return;
        }
        renderer.push_vnode(rect, "HuginHoverCard");

        if let Some(content) = &self.content {
            let _size = content.intrinsic_size(renderer, SizeProposal::unspecified());
            // Draw card background with shadow
            renderer.push_shadow(12.0, theme::shadow(), [0.0, 4.0]);
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(rect, 8.0, 1.5, 0.9);
            }
            renderer.stroke_rounded_rect(rect, RADIUS_LG, theme::border(), 1.0);
            renderer.pop_shadow();

            content.render(renderer, rect.inset(8.0));
        }

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        if !self.is_active {
            return Size {
                width: 0.0,
                height: 0.0,
            };
        }
        if let Some(content) = &self.content {
            let size = content.intrinsic_size(renderer, proposal);
            Size {
                width: size.width + 16.0,
                height: size.height + 16.0,
            }
        } else {
            Size {
                width: 0.0,
                height: 0.0,
            }
        }
    }
}

// Display & Utility
#[derive(Clone, Default)]
pub struct SkollTimeline<V> {
    items: Vec<V>,
}

impl<V: View> SkollTimeline<V> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(mut self, item: V) -> Self {
        self.items.push(item);
        self
    }
}

impl<V: View> View for SkollTimeline<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SkollTimeline");

        let mut y_offset = 0.0;
        let spacing = 16.0;
        let dot_radius = 6.0;
        let line_x = rect.x + 16.0;
        let content_x = line_x + 24.0;
        let content_w = (rect.width - 40.0).max(0.0);

        let item_count = self.items.len();

        for (i, item) in self.items.iter().enumerate() {
            let item_size = item.intrinsic_size(renderer, SizeProposal::width(content_w));
            let content_rect = Rect::new(content_x, rect.y + y_offset, content_w, item_size.height);

            let dot_y = rect.y + y_offset + 12.0; // Align dot with the first line of content

            // Draw connecting line to the next item
            if i < item_count - 1 {
                let _next_item_size =
                    self.items[i + 1].intrinsic_size(renderer, SizeProposal::width(content_w));
                let next_dot_y = rect.y + y_offset + item_size.height + spacing + 12.0;
                renderer.draw_line(line_x, dot_y, line_x, next_dot_y, theme::border(), 2.0);
            }

            // Draw dot
            renderer.fill_ellipse(
                Rect::new(
                    line_x - dot_radius,
                    dot_y - dot_radius,
                    dot_radius * 2.0,
                    dot_radius * 2.0,
                ),
                theme::accent(),
            );

            // Render content
            item.render(renderer, content_rect);

            y_offset += item_size.height + spacing;
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut total_height = 0.0;
        let mut max_width = 0.0_f32;
        let spacing = 16.0;
        let content_w = (proposal.width.unwrap_or(300.0) - 40.0).max(0.0);

        for item in &self.items {
            let item_size = item.intrinsic_size(renderer, SizeProposal::width(content_w));
            total_height += item_size.height + spacing;
            max_width = max_width.max(item_size.width);
        }

        Size {
            width: max_width + 40.0,
            height: (total_height - spacing).max(0.0),
        }
    }
}

#[derive(Clone)]
pub struct NidhugMasonry<V> {
    items: Vec<V>,
    columns: usize,
}

impl<V> NidhugMasonry<V> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            columns: 2,
        } // default to 2 columns
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    pub fn item(mut self, item: V) -> Self {
        self.items.push(item);
        self
    }
}

impl<V> Default for NidhugMasonry<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for NidhugMasonry<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NidhugMasonry");

        let spacing = 16.0;
        let col_width =
            ((rect.width - (self.columns - 1) as f32 * spacing) / self.columns as f32).max(0.0);

        // Track the current Y offset for each column
        let mut column_heights = vec![0.0_f32; self.columns];

        for item in &self.items {
            // Find the shortest column
            let (col_idx, &min_h) = column_heights
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));

            let item_x = rect.x + col_idx as f32 * (col_width + spacing);
            let item_y = rect.y + min_h;

            let item_size = item.intrinsic_size(renderer, SizeProposal::width(col_width));
            let item_rect = Rect::new(item_x, item_y, col_width, item_size.height);

            // Render the item at the calculated position
            item.render(renderer, item_rect);

            // Update column height
            column_heights[col_idx] += item_size.height + spacing;
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let spacing = 16.0;
        let width = proposal.width.unwrap_or(400.0);
        let col_width =
            ((width - (self.columns - 1) as f32 * spacing) / self.columns as f32).max(0.0);

        let mut column_heights = vec![0.0_f32; self.columns];

        for item in &self.items {
            let (col_idx, &_min_h) = column_heights
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));

            let item_size = item.intrinsic_size(renderer, SizeProposal::width(col_width));
            column_heights[col_idx] += item_size.height + spacing;
        }

        let max_height = column_heights
            .into_iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        Size {
            width,
            height: (max_height - spacing).max(0.0),
        }
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }
}

impl<V: View> cvkg_core::layout::LayoutView for NidhugMasonry<V> {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        let spacing = 16.0;
        let width = proposal.width.unwrap_or(400.0);
        let col_width =
            ((width - (self.columns - 1) as f32 * spacing) / self.columns as f32).max(0.0);

        let mut column_heights = vec![0.0_f32; self.columns];

        for item in &self.items {
            let (col_idx, _) = column_heights
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));

            let item_size = if let Some(l) = item.layout() {
                l.size_that_fits(
                    cvkg_core::layout::SizeProposal::width(col_width),
                    &[],
                    cache,
                )
            } else {
                cvkg_core::Size::ZERO
            };

            column_heights[col_idx] += item_size.height + spacing;
        }

        let max_height = column_heights
            .into_iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        cvkg_core::Size {
            width,
            height: (max_height - spacing).max(0.0),
        }
    }

    fn place_subviews(
        &self,
        bounds: cvkg_core::Rect,
        subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        let n = subviews.len();
        if n == 0 {
            return;
        }
        let spacing = 16.0;
        let columns = self.columns;
        let col_width = ((bounds.width - (columns - 1) as f32 * spacing) / columns as f32).max(0.0);

        // Measure all subviews first
        let mut child_sizes: Vec<cvkg_core::Size> = Vec::with_capacity(n);
        for sv in subviews.iter() {
            let sz = sv.size_that_fits(
                cvkg_core::layout::SizeProposal::width(col_width),
                &[],
                cache,
            );
            child_sizes.push(sz);
        }

        // Bin-packing: assign to shortest column
        let mut column_heights = vec![0.0_f32; columns];
        for (i, sv) in subviews.iter_mut().enumerate() {
            let (col_idx, &min_h) = column_heights
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            let item_x = bounds.x + col_idx as f32 * (col_width + spacing);
            let item_y = bounds.y + min_h;
            let child_h = child_sizes[i].height;
            let child_rect = cvkg_core::Rect::new(item_x, item_y, col_width, child_h);
            sv.place_subviews(child_rect, &mut [], cache);
            column_heights[col_idx] += child_h + spacing;
        }
    }
}

#[derive(Clone)]
pub struct VedrHero<V> {
    content: Option<V>,
}
impl<V> VedrHero<V> {
    pub fn new() -> Self {
        Self { content: None }
    }
    pub fn content(mut self, c: V) -> Self {
        self.content = Some(c);
        self
    }
}
impl<V> Default for VedrHero<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for VedrHero<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "VedrHero");

        // Vibrant hero background
        renderer.fill_rect(rect, theme::surface_elevated());
        // Hero visual accent (glowing orb behind content)
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(rect, 15.0, 1.0, 0.5);
        }
        renderer.fill_ellipse(
            Rect::new(
                rect.x + rect.width / 2.0 - 150.0,
                rect.y + rect.height / 2.0 - 150.0,
                300.0,
                300.0,
            ),
            theme::with_alpha(theme::accent(), 0.2),
        );

        if let Some(content) = &self.content {
            let size = content.intrinsic_size(
                renderer,
                SizeProposal::tight(rect.width * 0.8, rect.height * 0.8),
            );
            let cx = rect.x + (rect.width - size.width) / 2.0;
            let cy = rect.y + (rect.height - size.height) / 2.0;
            content.render(renderer, Rect::new(cx, cy, size.width, size.height));
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        // Hero takes full available space, minimum 300 height
        Size {
            width: proposal.width.unwrap_or(800.0),
            height: proposal.height.unwrap_or(400.0).max(300.0),
        }
    }
}

#[derive(Clone, Default)]
pub struct GullinStat<V> {
    #[allow(dead_code)]
    content: Option<V>,
}
impl<V: View> View for GullinStat<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Clone, Default)]
pub struct RatatoKey<V> {
    #[allow(dead_code)]
    content: Option<V>,
}
impl<V: View> View for RatatoKey<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

// Navigation
#[derive(Clone)]
pub struct GraniBreadcrumb<V> {
    crumbs: Vec<V>,
}
impl<V> GraniBreadcrumb<V> {
    pub fn new() -> Self {
        Self { crumbs: Vec::new() }
    }
    pub fn crumb(mut self, item: V) -> Self {
        self.crumbs.push(item);
        self
    }
}
impl<V> Default for GraniBreadcrumb<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for GraniBreadcrumb<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GraniBreadcrumb");
        let mut x_offset = 0.0;
        let spacing = 8.0;

        for (i, crumb) in self.crumbs.iter().enumerate() {
            let size = crumb.intrinsic_size(renderer, SizeProposal::unspecified());
            crumb.render(
                renderer,
                Rect::new(rect.x + x_offset, rect.y, size.width, size.height),
            );
            x_offset += size.width + spacing;

            if i < self.crumbs.len() - 1 {
                renderer.draw_text(
                    "/",
                    rect.x + x_offset,
                    rect.y + size.height * 0.7,
                    14.0,
                    theme::text_muted(),
                );
                x_offset += 10.0 + spacing;
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let mut total_width = 0.0;
        let mut max_height = 0.0_f32;
        let spacing = 8.0;

        for (i, crumb) in self.crumbs.iter().enumerate() {
            let size = crumb.intrinsic_size(renderer, SizeProposal::unspecified());
            total_width += size.width + spacing;
            max_height = max_height.max(size.height);
            if i < self.crumbs.len() - 1 {
                total_width += 10.0 + spacing;
            }
        }
        Size {
            width: (total_width - spacing).max(0.0),
            height: max_height,
        }
    }
}

#[derive(Clone)]
pub struct FrekiBottomNav<V> {
    items: Vec<V>,
}
impl<V> FrekiBottomNav<V> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn item(mut self, item: V) -> Self {
        self.items.push(item);
        self
    }
}
impl<V> Default for FrekiBottomNav<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for FrekiBottomNav<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FrekiBottomNav");
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(rect, 0.0, 2.0, 0.9); // Frosted bar
        }
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            theme::border(),
            1.0,
        ); // Top border

        let count = self.items.len();
        if count > 0 {
            let item_width = rect.width / count as f32;
            for (i, item) in self.items.iter().enumerate() {
                let item_rect = Rect::new(
                    rect.x + (i as f32) * item_width,
                    rect.y,
                    item_width,
                    rect.height,
                );
                item.render(renderer, item_rect.inset(8.0));
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(400.0),
            height: 60.0,
        }
    }
}

#[derive(Clone)]
pub struct GarmSpeedDial<V> {
    items: Vec<V>,
    is_open: bool,
}
impl<V> GarmSpeedDial<V> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            is_open: false,
        }
    }
    pub fn item(mut self, item: V) -> Self {
        self.items.push(item);
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }
}
impl<V> Default for GarmSpeedDial<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for GarmSpeedDial<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GarmSpeedDial");

        let main_radius = 28.0;
        let spacing = 12.0;

        // Main button at bottom
        let main_rect = Rect::new(
            rect.x + rect.width / 2.0 - main_radius,
            rect.y + rect.height - main_radius * 2.0,
            main_radius * 2.0,
            main_radius * 2.0,
        );
        renderer.fill_ellipse(main_rect, theme::primary());
        renderer.stroke_ellipse(main_rect, theme::with_alpha(theme::primary(), 0.8), 2.0);

        // Draw cross icon
        let cx = main_rect.x + main_radius;
        let cy = main_rect.y + main_radius;
        renderer.draw_line(cx - 10.0, cy, cx + 10.0, cy, theme::text(), 2.0);
        if !self.is_open {
            renderer.draw_line(cx, cy - 10.0, cx, cy + 10.0, theme::text(), 2.0);
        }

        if self.is_open {
            let mut current_y = main_rect.y - spacing;
            for item in self.items.iter().rev() {
                let size = item.intrinsic_size(renderer, SizeProposal::unspecified());
                let item_rect = Rect::new(
                    rect.x + rect.width / 2.0 - size.width / 2.0,
                    current_y - size.height,
                    size.width,
                    size.height,
                );
                item.render(renderer, item_rect);
                current_y -= size.height + spacing;
            }
        }

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let main_radius = 28.0;
        let spacing = 12.0;
        let mut total_height = main_radius * 2.0;
        let mut max_width = main_radius * 2.0;

        if self.is_open {
            for item in &self.items {
                let size = item.intrinsic_size(renderer, SizeProposal::unspecified());
                total_height += size.height + spacing;
                max_width = max_width.max(size.width);
            }
        }
        Size {
            width: max_width,
            height: total_height,
        }
    }
}

#[derive(Clone, Default)]
pub struct HuginContextMenu<V> {
    #[allow(dead_code)]
    items: Vec<V>,
}
impl<V: View> View for HuginContextMenu<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Clone, Default)]
pub struct MuninMenubar<V> {
    items: Vec<V>,
}
impl<V: View> View for MuninMenubar<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MuninMenubar");
        renderer.fill_rect(rect, theme::surface());

        let layouts: Vec<&dyn cvkg_core::layout::LayoutView> =
            self.items.iter().filter_map(|c| c.layout()).collect();
        let mut cache = cvkg_core::layout::LayoutCache::new();
        let inner_rect = Rect::new(rect.x + 8.0, rect.y, rect.width - 16.0, rect.height);

        // Delegate structural geometry to the layout engine
        let rects = cvkg_layout::HStack::compute_layout(
            12.0,
            cvkg_core::Alignment::Center,
            cvkg_core::Distribution::Leading,
            inner_rect,
            &layouts,
            &mut cache,
        );

        let mut idx = 0;
        for item in &self.items {
            if item.layout().is_some() && idx < rects.len() {
                item.render(renderer, rects[idx]);
                idx += 1;
            }
        }
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let layouts: Vec<&dyn cvkg_core::layout::LayoutView> =
            self.items.iter().filter_map(|c| c.layout()).collect();
        let mut cache = cvkg_core::layout::LayoutCache::new();
        let mut w = 16.0;
        for l in layouts {
            w += l
                .size_that_fits(SizeProposal::unspecified(), &[], &mut cache)
                .width
                + 12.0;
        }
        Size {
            width: proposal.width.unwrap_or(w),
            height: proposal.height.unwrap_or(32.0),
        }
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }
}

impl<V: View> cvkg_core::layout::LayoutView for MuninMenubar<V> {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        let layouts: Vec<&dyn cvkg_core::layout::LayoutView> =
            self.items.iter().filter_map(|c| c.layout()).collect();
        let mut w = 16.0;
        for l in layouts {
            w += l
                .size_that_fits(cvkg_core::layout::SizeProposal::unspecified(), &[], cache)
                .width
                + 12.0;
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(w),
            height: proposal.height.unwrap_or(32.0),
        }
    }

    fn place_subviews(
        &self,
        bounds: cvkg_core::Rect,
        subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        let layouts: Vec<&dyn cvkg_core::layout::LayoutView> =
            self.items.iter().filter_map(|c| c.layout()).collect();
        if layouts.is_empty() {
            return;
        }
        let inner_rect =
            cvkg_core::Rect::new(bounds.x + 8.0, bounds.y, bounds.width - 16.0, bounds.height);
        let rects = cvkg_layout::HStack::compute_layout(
            12.0,
            cvkg_core::Alignment::Center,
            cvkg_core::Distribution::Leading,
            inner_rect,
            &layouts,
            cache,
        );
        // Delegate each subview into its computed rect
        for (i, sv) in subviews.iter_mut().enumerate() {
            if i < rects.len() {
                sv.place_subviews(rects[i], &mut [], cache);
            }
        }
    }
}

#[derive(Clone)]
pub struct MuninSpy<V> {
    items: Vec<V>,
    active_index: usize,
}
impl<V> MuninSpy<V> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active_index: 0,
        }
    }
    pub fn item(mut self, item: V) -> Self {
        self.items.push(item);
        self
    }
    pub fn active(mut self, idx: usize) -> Self {
        self.active_index = idx;
        self
    }
}
impl<V> Default for MuninSpy<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for MuninSpy<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MuninSpy");
        let mut y_offset = 0.0;
        let spacing = 8.0;
        let content_x = rect.x + 12.0;

        // Draw track line
        renderer.draw_line(
            rect.x + 2.0,
            rect.y,
            rect.x + 2.0,
            rect.y + rect.height,
            theme::border(),
            2.0,
        );

        for (i, item) in self.items.iter().enumerate() {
            let size = item.intrinsic_size(renderer, SizeProposal::width(rect.width - 12.0));

            if i == self.active_index {
                // Draw active indicator bar
                renderer.draw_line(
                    rect.x + 2.0,
                    rect.y + y_offset,
                    rect.x + 2.0,
                    rect.y + y_offset + size.height,
                    theme::accent(),
                    4.0,
                );
            }

            item.render(
                renderer,
                Rect::new(content_x, rect.y + y_offset, size.width, size.height),
            );
            y_offset += size.height + spacing;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut total_height = 0.0;
        let mut max_width = 0.0_f32;
        let spacing = 8.0;

        for item in &self.items {
            let size = item.intrinsic_size(
                renderer,
                SizeProposal::width(proposal.width.unwrap_or(200.0) - 12.0),
            );
            total_height += size.height + spacing;
            max_width = max_width.max(size.width);
        }
        Size {
            width: max_width + 12.0,
            height: (total_height - spacing).max(0.0),
        }
    }
}

// Advanced Data & Input
#[derive(Clone, Default)]
pub struct SkollTime<V> {
    #[allow(dead_code)]
    content: Option<V>,
}
impl<V: View> View for SkollTime<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Clone, Default)]
pub struct HatiSwipe<V> {
    #[allow(dead_code)]
    content: Option<V>,
}
impl<V: View> View for HatiSwipe<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct FafnirOTP {
    length: usize,
    value: String,
}
impl FafnirOTP {
    pub fn new() -> Self {
        Self {
            length: 6,
            value: String::new(),
        }
    }
    pub fn length(mut self, l: usize) -> Self {
        self.length = l;
        self
    }
    pub fn value(mut self, v: &str) -> Self {
        self.value = v.to_string();
        self
    }
}
impl Default for FafnirOTP {
    fn default() -> Self {
        Self::new()
    }
}

impl View for FafnirOTP {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FafnirOTP");

        let box_size = 48.0;
        let spacing = 12.0;
        let chars: Vec<char> = self.value.chars().collect();

        for i in 0..self.length {
            let box_x = rect.x + (i as f32) * (box_size + spacing);
            let box_rect = Rect::new(box_x, rect.y, box_size, box_size);

            renderer.fill_rounded_rect(box_rect, RADIUS_MD, theme::surface_elevated());

            // Highlight focused/next box
            if i == chars.len() {
                renderer.stroke_rounded_rect(box_rect, RADIUS_MD, theme::focus_ring(), 2.0);
            } else {
                renderer.stroke_rounded_rect(box_rect, RADIUS_MD, theme::border(), 1.0);
            }

            if let Some(&c) = chars.get(i) {
                renderer.draw_text(
                    &c.to_string(),
                    box_rect.x + box_size / 2.0 - 6.0,
                    box_rect.y + box_size / 2.0 + 6.0,
                    24.0,
                    theme::text(),
                );
            }
        }

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let box_size = 48.0;
        let spacing = 12.0;
        let total_width = (self.length as f32) * box_size + ((self.length - 1) as f32) * spacing;
        Size {
            width: total_width,
            height: box_size,
        }
    }
}

#[derive(Clone)]
pub struct NidhugSplitter<L, R> {
    left: Option<L>,
    right: Option<R>,
    ratio: f32,
}
impl<L, R> NidhugSplitter<L, R> {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
            ratio: 0.5,
        }
    }
    pub fn left(mut self, l: L) -> Self {
        self.left = Some(l);
        self
    }
    pub fn right(mut self, r: R) -> Self {
        self.right = Some(r);
        self
    }
    pub fn ratio(mut self, r: f32) -> Self {
        self.ratio = r.clamp(0.1, 0.9);
        self
    }
}
impl<L, R> Default for NidhugSplitter<L, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: View, R: View> View for NidhugSplitter<L, R> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NidhugSplitter");

        let split_x = rect.x + rect.width * self.ratio;
        let divider_width = 4.0;

        if let Some(l) = &self.left {
            let l_rect = Rect::new(
                rect.x,
                rect.y,
                rect.width * self.ratio - divider_width / 2.0,
                rect.height,
            );
            l.render(renderer, l_rect);
        }

        if let Some(r) = &self.right {
            let r_rect = Rect::new(
                split_x + divider_width / 2.0,
                rect.y,
                rect.width * (1.0 - self.ratio) - divider_width / 2.0,
                rect.height,
            );
            r.render(renderer, r_rect);
        }

        // Render Splitter Handle
        let handle_rect = Rect::new(
            split_x - divider_width / 2.0,
            rect.y,
            divider_width,
            rect.height,
        );
        renderer.fill_rect(handle_rect, theme::border());

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        // Typically a splitter fills available space
        Size {
            width: proposal.width.unwrap_or(800.0),
            height: proposal.height.unwrap_or(400.0),
        }
    }
}
