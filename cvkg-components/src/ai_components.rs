use cvkg_core::{View, Renderer, Rect, Size, SizeProposal, Never};
use crate::theme;
use std::sync::Arc;

// ----------------------------------------------------------------------------
// Batch 1: Inputs & Generative Formatting
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct GeriPrompt<V> { text: String, attachments: Vec<V> }
impl<V> GeriPrompt<V> {
    pub fn new() -> Self { Self { text: String::new(), attachments: Vec::new() } }
    pub fn text(mut self, t: &str) -> Self { self.text = t.to_string(); self }
    pub fn attachment(mut self, a: V) -> Self { self.attachments.push(a); self }
}
impl<V> Default for GeriPrompt<V> { fn default() -> Self { Self::new() } }

impl<V: View> View for GeriPrompt<V> {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GeriPrompt");
        renderer.push_shadow(10.0, [0.0, 0.0, 0.0, 0.2], [0.0, 4.0]);
        renderer.fill_rounded_rect(rect, 12.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 12.0, theme::border_strong(), 1.0);
        renderer.pop_shadow();
        
        let mut current_y = rect.y + 12.0;
        let padding_x = 16.0;
        
        // Render Attachments Flow
        if !self.attachments.is_empty() {
            let mut current_x = rect.x + padding_x;
            for attachment in &self.attachments {
                let size = attachment.intrinsic_size(renderer, SizeProposal::unspecified());
                // Wrap to next line if needed
                if current_x + size.width > rect.x + rect.width - padding_x {
                    current_x = rect.x + padding_x;
                    current_y += size.height + 8.0;
                }
                attachment.render(renderer, Rect::new(current_x, current_y, size.width, size.height));
                current_x += size.width + 8.0;
            }
            current_y += 40.0; // Spacing below attachments
        }
        
        // Render Text Input Placeholder
        let text_rect = Rect::new(rect.x + padding_x, current_y, rect.width - 60.0, 24.0);
        if self.text.is_empty() {
            renderer.draw_text("Send a message...", text_rect.x, text_rect.y + 16.0, 16.0, theme::text_muted());
        } else {
            renderer.draw_text(&self.text, text_rect.x, text_rect.y + 16.0, 16.0, theme::text());
        }
        
        // Render Send Button
        let send_btn_rect = Rect::new(rect.x + rect.width - 44.0, rect.y + rect.height - 44.0, 32.0, 32.0);
        renderer.fill_rounded_rect(send_btn_rect, 8.0, theme::accent());
        // Simple up arrow for send
        let arrow_col = theme::bg();
        renderer.draw_line(send_btn_rect.x + 16.0, send_btn_rect.y + 22.0, send_btn_rect.x + 16.0, send_btn_rect.y + 10.0, arrow_col, 2.0);
        renderer.draw_line(send_btn_rect.x + 16.0, send_btn_rect.y + 10.0, send_btn_rect.x + 10.0, send_btn_rect.y + 16.0, arrow_col, 2.0);
        renderer.draw_line(send_btn_rect.x + 16.0, send_btn_rect.y + 10.0, send_btn_rect.x + 22.0, send_btn_rect.y + 16.0, arrow_col, 2.0);
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(600.0);
        let mut height = 24.0 + 24.0; // Text area + padding
        
        if !self.attachments.is_empty() {
            let mut current_x = 16.0;
            let mut row_h = 0.0_f32;
            for attachment in &self.attachments {
                let size = attachment.intrinsic_size(renderer, SizeProposal::unspecified());
                if current_x + size.width > width - 16.0 {
                    height += row_h + 8.0;
                    current_x = 16.0;
                    row_h = 0.0;
                }
                row_h = row_h.max(size.height);
                current_x += size.width + 8.0;
            }
            height += row_h + 16.0;
        }
        
        Size { width, height: height.max(56.0) }
    }
}

#[derive(Clone)]
pub struct HuginGhost { text: String }
impl HuginGhost {
    pub fn new() -> Self { Self { text: String::new() } }
    pub fn ghost(mut self, text: &str) -> Self { self.text = text.to_string(); self }
}
impl Default for HuginGhost { fn default() -> Self { Self::new() } }
impl View for HuginGhost {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HuginGhost");
        renderer.draw_text(&self.text, rect.x, rect.y + 16.0, 16.0, theme::text_dim());
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: (self.text.len() as f32 * 9.0), height: 24.0 }
    }
}

#[derive(Clone)]
pub struct HuginChat { role: String, message: String }
impl HuginChat {
    pub fn new() -> Self { Self { role: "assistant".to_string(), message: String::new() } }
    pub fn message(mut self, role: &str, msg: &str) -> Self { self.role = role.to_string(); self.message = msg.to_string(); self }
}
impl Default for HuginChat { fn default() -> Self { Self::new() } }
impl View for HuginChat {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HuginChat");
        let is_user = self.role == "user";
        let bg_color = if is_user { theme::chat_bubble_user() } else { theme::chat_bubble_assistant() };
        
        renderer.fill_rounded_rect(rect, 12.0, bg_color);
        if !is_user {
            renderer.stroke_rounded_rect(rect, 12.0, theme::border(), 1.0);
        }
        
        let header_str = if is_user { "You" } else { "AI Assistant" };
        let header_color = if is_user { theme::bg() } else { theme::success() };
        renderer.draw_text(header_str, rect.x + 16.0, rect.y + 24.0, 12.0, header_color);
        
        let msg_color = if is_user { theme::chat_text_user() } else { theme::chat_text_assistant() };
        renderer.draw_text(&self.message, rect.x + 16.0, rect.y + 48.0, 15.0, msg_color);
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(500.0);
        let lines = (self.message.len() as f32 * 8.0 / (width - 32.0)).ceil().max(1.0);
        Size { width, height: 40.0 + lines * 20.0 + 16.0 }
    }
}

#[derive(Clone)]
pub struct DvalinMedia { filename: String, is_image: bool }
impl DvalinMedia {
    pub fn new() -> Self { Self { filename: String::new(), is_image: false } }
    pub fn file(mut self, name: &str, is_img: bool) -> Self { self.filename = name.to_string(); self.is_image = is_img; self }
}
impl Default for DvalinMedia { fn default() -> Self { Self::new() } }

impl View for DvalinMedia {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DvalinMedia");
        renderer.fill_rounded_rect(rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        
        let icon_rect = Rect::new(rect.x + 8.0, rect.y + 8.0, 32.0, 32.0);
        if self.is_image {
            renderer.fill_rounded_rect(icon_rect, 4.0, theme::hover());
            renderer.stroke_rounded_rect(icon_rect, 4.0, theme::accent(), 1.0);
        } else {
            renderer.fill_rounded_rect(icon_rect, 4.0, theme::hover());
            renderer.stroke_rounded_rect(icon_rect, 4.0, theme::error_color(), 1.0);
        }
        
        renderer.draw_text(&self.filename, rect.x + 48.0, rect.y + 28.0, 14.0, theme::text());
        
        // Remove 'X' icon
        let close_x = rect.x + rect.width - 20.0;
        let close_y = rect.y + 24.0;
        let cross_col = theme::text_muted();
        renderer.draw_line(close_x - 4.0, close_y - 4.0, close_x + 4.0, close_y + 4.0, cross_col, 2.0);
        renderer.draw_line(close_x + 4.0, close_y - 4.0, close_x - 4.0, close_y + 4.0, cross_col, 2.0);
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        // Approximate width based on filename length
        Size { width: 80.0 + (self.filename.len() as f32 * 8.0).min(150.0), height: 48.0 }
    }
}

#[derive(Clone)]
pub struct GullinSnip { language: String, lines: usize }
impl GullinSnip {
    pub fn new() -> Self { Self { language: "text".to_string(), lines: 0 } }
    pub fn info(mut self, lang: &str, lines: usize) -> Self { self.language = lang.to_string(); self.lines = lines; self }
}
impl Default for GullinSnip { fn default() -> Self { Self::new() } }

impl View for GullinSnip {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GullinSnip");
        renderer.fill_rounded_rect(rect, 6.0, theme::code_bg());
        renderer.stroke_rounded_rect(rect, 6.0, theme::border(), 1.0);
        
        let header_str = format!("Pasted Code ({} - {} lines)", self.language, self.lines);
        renderer.draw_text(&header_str, rect.x + 12.0, rect.y + 24.0, 14.0, theme::text_dim());
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 300.0, height: 40.0 }
    }
}

#[derive(Clone)]
pub struct HatiStream { text: String, is_streaming: bool }
impl HatiStream {
    pub fn new() -> Self { Self { text: String::new(), is_streaming: false } }
    pub fn stream(mut self, t: &str, streaming: bool) -> Self { self.text = t.to_string(); self.is_streaming = streaming; self }
}
impl Default for HatiStream { fn default() -> Self { Self::new() } }

impl View for HatiStream {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HatiStream");
        // Simulated markdown/text wrapping rendering
        renderer.draw_text(&self.text, rect.x, rect.y + 16.0, 16.0, theme::text());
        
        if self.is_streaming {
            // Draw blinking block cursor at the end
            let estimated_w = (self.text.len() as f32 * 9.0).min(rect.width - 10.0);
            let lines = (self.text.len() as f32 * 9.0 / rect.width).floor();
            let cursor_x = rect.x + estimated_w % rect.width;
            let cursor_y = rect.y + lines * 22.0 + 4.0;
            renderer.fill_rect(Rect::new(cursor_x + 4.0, cursor_y, 8.0, 16.0), theme::accent());
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(400.0);
        let lines = (self.text.len() as f32 * 9.0 / width).ceil().max(1.0);
        Size { width, height: lines * 22.0 + 16.0 }
    }
}

#[derive(Clone)]
pub struct FenrirCode { 
    code: String, 
    language: String,
    on_copy: Option<Arc<dyn Fn() + Send + Sync>> 
}
impl FenrirCode {
    pub fn new() -> Self { Self { code: String::new(), language: String::new(), on_copy: None } }
    pub fn block(mut self, code: &str, lang: &str) -> Self { self.code = code.to_string(); self.language = lang.to_string(); self }
    pub fn on_copy(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self { self.on_copy = Some(Arc::new(handler)); self }
}
impl Default for FenrirCode { fn default() -> Self { Self::new() } }

impl View for FenrirCode {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FenrirCode");
        
        // Container
        renderer.fill_rounded_rect(rect, 8.0, theme::code_bg());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        
        // Top Bar
        let bar_rect = Rect::new(rect.x, rect.y, rect.width, 32.0);
        renderer.fill_rounded_rect(bar_rect, 8.0, theme::surface_elevated()); // Darker top bar
        // Hide bottom rounded corners of the bar by drawing a sharp rect over the bottom half
        renderer.fill_rect(Rect::new(rect.x, rect.y + 16.0, rect.width, 16.0), theme::surface_elevated());
        renderer.draw_line(rect.x, rect.y + 32.0, rect.x + rect.width, rect.y + 32.0, theme::border(), 1.0);
        
        // Language label
        let lang_str = if self.language.is_empty() { "text" } else { &self.language };
        renderer.draw_text(lang_str, rect.x + 16.0, rect.y + 22.0, 14.0, theme::text_dim());
        
        // Copy Button
        let copy_x = rect.x + rect.width - 60.0;
        renderer.draw_text("Copy", copy_x, rect.y + 22.0, 13.0, theme::text_muted());
        
        if let Some(handler) = &self.on_copy {
            let h = handler.clone();
            let copy_btn_rect = Rect::new(copy_x - 10.0, rect.y + 8.0, 60.0, 24.0);
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |ev| {
                    if let cvkg_core::Event::PointerClick { x, y, .. } = ev {
                        if x >= copy_btn_rect.x && x <= copy_btn_rect.x + copy_btn_rect.width &&
                           y >= copy_btn_rect.y && y <= copy_btn_rect.y + copy_btn_rect.height {
                            h();
                        }
                    }
                })
            );
        }
        
        // Code content
        renderer.draw_text(&self.code, rect.x + 16.0, rect.y + 56.0, 14.0, theme::text());
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(600.0);
        let lines = self.code.lines().count().max(1) as f32;
        Size { width, height: 48.0 + lines * 20.0 + 16.0 }
    }
}

// ----------------------------------------------------------------------------
// Batch 2: Feedback & Orchestration Config
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct GraniRate { liked: Option<bool> }
impl GraniRate {
    pub fn new() -> Self { Self { liked: None } }
    pub fn state(mut self, liked: Option<bool>) -> Self { self.liked = liked; self }
}
impl Default for GraniRate { fn default() -> Self { Self::new() } }

impl View for GraniRate {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GraniRate");
        
        // Thumbs Up
        let up_color = if self.liked == Some(true) { theme::success() } else { theme::text_muted() };
        renderer.draw_text("Up", rect.x + 8.0, rect.y + 16.0, 14.0, up_color);
        
        // Thumbs Down
        let down_color = if self.liked == Some(false) { theme::error_color() } else { theme::text_muted() };
        renderer.draw_text("Down", rect.x + 40.0, rect.y + 16.0, 14.0, down_color);
        
        // Copy
        renderer.draw_text("Copy", rect.x + 90.0, rect.y + 16.0, 14.0, theme::text_muted());
        
        // Regenerate
        renderer.draw_text("Regen", rect.x + 140.0, rect.y + 16.0, 14.0, theme::text_muted());
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 200.0, height: 24.0 }
    }
}

#[derive(Clone)]
pub struct MuninPill { label: String, color: [f32; 4] }
impl MuninPill {
    pub fn new() -> Self { Self { label: String::new(), color: [0.2, 0.5, 0.9, 1.0] } }
    pub fn content(mut self, text: &str, col: [f32; 4]) -> Self { self.label = text.to_string(); self.color = col; self }
}
impl Default for MuninPill { fn default() -> Self { Self::new() } }

impl View for MuninPill {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MuninPill");
        renderer.fill_rounded_rect(rect, 12.0, [self.color[0], self.color[1], self.color[2], 0.2]);
        renderer.stroke_rounded_rect(rect, 12.0, self.color, 1.0);
        renderer.draw_text(&self.label, rect.x + 12.0, rect.y + 16.0, 14.0, theme::text());
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 24.0 + (self.label.len() as f32 * 8.0), height: 24.0 }
    }
}

#[derive(Clone)]
pub struct RatatoChip { label: String }
impl RatatoChip {
    pub fn new() -> Self { Self { label: String::new() } }
    pub fn label(mut self, text: &str) -> Self { self.label = text.to_string(); self }
}
impl Default for RatatoChip { fn default() -> Self { Self::new() } }
impl View for RatatoChip {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RatatoChip");
        renderer.fill_rounded_rect(rect, 16.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 16.0, theme::border(), 1.5);
        renderer.draw_text(&self.label, rect.x + 16.0, rect.y + 20.0, 14.0, theme::text());
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 32.0 + (self.label.len() as f32 * 8.0), height: 32.0 }
    }
}

#[derive(Clone)]
pub struct GeriSpark { active: bool }
impl GeriSpark {
    pub fn new() -> Self { Self { active: false } }
    pub fn state(mut self, active: bool) -> Self { self.active = active; self }
}
impl Default for GeriSpark { fn default() -> Self { Self::new() } }
impl View for GeriSpark {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GeriSpark");
        let bg_color = if self.active { theme::accent() } else { theme::surface_elevated() };
        
        renderer.push_shadow(8.0, [0.7, 0.3, 0.9, 0.4], [0.0, 4.0]);
        renderer.fill_ellipse(rect, bg_color);
        renderer.pop_shadow();
        
        // Sparkle icon
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        let spark_col = if self.active { theme::bg() } else { theme::text_muted() };
        renderer.draw_line(cx, cy - 8.0, cx, cy + 8.0, spark_col, 2.0);
        renderer.draw_line(cx - 8.0, cy, cx + 8.0, cy, spark_col, 2.0);
        
        if self.active {
            renderer.bifrost(Rect::new(rect.x - 4.0, rect.y - 4.0, rect.width + 8.0, rect.height + 8.0), 5.0, 1.0, 0.3);
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 40.0, height: 40.0 }
    }
}

#[derive(Clone)]
pub struct DainnSkill { name: String, enabled: bool }
impl DainnSkill {
    pub fn new() -> Self { Self { name: String::new(), enabled: false } }
    pub fn skill(mut self, name: &str, enabled: bool) -> Self { self.name = name.to_string(); self.enabled = enabled; self }
}
impl Default for DainnSkill { fn default() -> Self { Self::new() } }

impl View for DainnSkill {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DainnSkill");
        renderer.fill_rounded_rect(rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        
        // Icon
        renderer.fill_rounded_rect(Rect::new(rect.x + 12.0, rect.y + 12.0, 32.0, 32.0), 6.0, theme::hover());
        
        // Name
        renderer.draw_text(&self.name, rect.x + 56.0, rect.y + 26.0, 16.0, theme::text());
        
        // Toggle Switch
        let toggle_x = rect.x + rect.width - 50.0;
        let toggle_y = rect.y + 16.0;
        let toggle_color = if self.enabled { theme::success() } else { theme::disabled() };
        renderer.fill_rounded_rect(Rect::new(toggle_x, toggle_y, 36.0, 20.0), 10.0, toggle_color);
        let knob_x = if self.enabled { toggle_x + 18.0 } else { toggle_x + 2.0 };
        renderer.fill_ellipse(Rect::new(knob_x, toggle_y + 2.0, 16.0, 16.0), theme::bg());
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(300.0), height: 56.0 }
    }
}

#[derive(Clone)]
pub struct HeidruMask { persona: String }
impl HeidruMask {
    pub fn new() -> Self { Self { persona: String::new() } }
    pub fn persona(mut self, name: &str) -> Self { self.persona = name.to_string(); self }
}
impl Default for HeidruMask { fn default() -> Self { Self::new() } }

impl View for HeidruMask {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HeidruMask");
        renderer.fill_rounded_rect(rect, 24.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 24.0, theme::border(), 1.0);
        
        // Avatar circle
        renderer.fill_ellipse(Rect::new(rect.x + 8.0, rect.y + 8.0, 32.0, 32.0), theme::hover());
        
        // Persona Name
        let label = if self.persona.is_empty() { "Select Persona..." } else { &self.persona };
        renderer.draw_text(label, rect.x + 48.0, rect.y + 26.0, 16.0, theme::text());
        
        // Dropdown Chevron
        let ch_x = rect.x + rect.width - 24.0;
        let ch_y = rect.y + 22.0;
        let chev_col = theme::text_muted();
        renderer.draw_line(ch_x, ch_y, ch_x + 5.0, ch_y + 5.0, chev_col, 2.0);
        renderer.draw_line(ch_x + 5.0, ch_y + 5.0, ch_x + 10.0, ch_y, chev_col, 2.0);
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(240.0), height: 48.0 }
    }
}

// ----------------------------------------------------------------------------
// Batch 3: Workflow Visualization
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SleipnFlow<V> { nodes: Vec<V> }
impl<V> SleipnFlow<V> {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
    pub fn node(mut self, n: V) -> Self { self.nodes.push(n); self }
}
impl<V> Default for SleipnFlow<V> { fn default() -> Self { Self::new() } }

impl<V: View> View for SleipnFlow<V> {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SleipnFlow");
        let mut y_offset = 0.0;
        let padding = 20.0;
        let node_w = rect.width.min(400.0);
        let center_x = rect.x + node_w / 2.0;
        
        for (i, node) in self.nodes.iter().enumerate() {
            let size = node.intrinsic_size(renderer, SizeProposal::width(node_w));
            
            // Draw connecting line to next node
            if i < self.nodes.len() - 1 {
                renderer.draw_line(center_x, rect.y + y_offset + size.height, center_x, rect.y + y_offset + size.height + padding, theme::border_strong(), 2.0);
            }
            
            node.render(renderer, Rect::new(rect.x, rect.y + y_offset, size.width, size.height));
            y_offset += size.height + padding;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(400.0);
        let mut total_h = 0.0;
        for node in &self.nodes {
            let size = node.intrinsic_size(renderer, SizeProposal::width(width));
            total_h += size.height + 20.0;
        }
        Size { width, height: (total_h - 20.0).max(0.0) }
    }
}

#[derive(Clone)]
pub struct FenrirNode { name: String, status: u8 } // 0=queued, 1=running, 2=done, 3=error
impl FenrirNode {
    pub fn new() -> Self { Self { name: String::new(), status: 0 } }
    pub fn info(mut self, name: &str, status: u8) -> Self { self.name = name.to_string(); self.status = status; self }
}
impl Default for FenrirNode { fn default() -> Self { Self::new() } }

impl View for FenrirNode {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FenrirNode");
        
        let bg_color = match self.status {
            0 => theme::surface_elevated(), // Queued
            1 => theme::hover(), // Running
            2 => theme::surface_elevated(), // Done
            3 => theme::surface_elevated(), // Error
            _ => theme::surface(),
        };
        let border_color = match self.status {
            0 => theme::border(),
            1 => theme::accent(),
            2 => theme::success(),
            3 => theme::error_color(),
            _ => theme::border(),
        };
        
        renderer.fill_rounded_rect(rect, 8.0, bg_color);
        renderer.stroke_rounded_rect(rect, 8.0, border_color, 2.0);
        
        // Status Indicator
        let icon_rect = Rect::new(rect.x + 16.0, rect.y + 16.0, 16.0, 16.0);
        if self.status == 1 {
            // "Running" - use a glowing ellipse
            renderer.fill_ellipse(icon_rect, theme::accent());
            renderer.bifrost(Rect::new(icon_rect.x - 8.0, icon_rect.y - 8.0, 32.0, 32.0), 5.0, 1.0, 0.2);
        } else {
            renderer.fill_ellipse(icon_rect, border_color);
        }
        
        renderer.draw_text(&self.name, rect.x + 48.0, rect.y + 28.0, 16.0, theme::text());
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(300.0), height: 48.0 }
    }
}

#[derive(Clone)]
pub struct SkollPulse { active: bool }
impl SkollPulse {
    pub fn new() -> Self { Self { active: false } }
    pub fn active(mut self, a: bool) -> Self { self.active = a; self }
}
impl Default for SkollPulse { fn default() -> Self { Self::new() } }

impl View for SkollPulse {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SkollPulse");
        if self.active {
            // Core
            renderer.fill_ellipse(rect, theme::accent());
            // Glow layer using bifrost
            renderer.bifrost(Rect::new(rect.x - 10.0, rect.y - 10.0, rect.width + 20.0, rect.height + 20.0), 8.0, 1.0, 0.5);
            // Outer pulse ring
            let mut pulse_ring = theme::accent();
            pulse_ring[3] = 0.5;
            renderer.stroke_ellipse(Rect::new(rect.x - 4.0, rect.y - 4.0, rect.width + 8.0, rect.height + 8.0), pulse_ring, 2.0);
        } else {
            // Dormant state
            renderer.fill_ellipse(rect, theme::disabled());
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 16.0, height: 16.0 }
    }
}

#[derive(Clone)]
pub struct FrekiDrawer<V> { open: bool, title: String, content: Option<V> }
impl<V> FrekiDrawer<V> {
    pub fn new() -> Self { Self { open: false, title: "Copilot".to_string(), content: None } }
    pub fn state(mut self, open: bool) -> Self { self.open = open; self }
    pub fn title(mut self, title: &str) -> Self { self.title = title.to_string(); self }
    pub fn content(mut self, c: V) -> Self { self.content = Some(c); self }
}
impl<V> Default for FrekiDrawer<V> { fn default() -> Self { Self::new() } }
impl<V: View> View for FrekiDrawer<V> {
    type Body = Never; fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.open { return; }
        
        renderer.push_vnode(rect, "FrekiDrawer");
        renderer.push_shadow(16.0, [0.0, 0.0, 0.0, 0.5], [-4.0, 0.0]);
        renderer.fill_rect(rect, theme::surface()); // Dark drawer background
        renderer.pop_shadow();
        
        // Border line on left side
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height, theme::border(), 1.0);
        
        // Header
        renderer.fill_rect(Rect::new(rect.x, rect.y, rect.width, 48.0), theme::surface_elevated());
        renderer.draw_text(&self.title, rect.x + 16.0, rect.y + 30.0, 16.0, theme::text());
        
        // Close Icon
        let close_x = rect.x + rect.width - 24.0;
        let close_y = rect.y + 24.0;
        let close_col = theme::text_muted();
        renderer.draw_line(close_x - 6.0, close_y - 6.0, close_x + 6.0, close_y + 6.0, close_col, 2.0);
        renderer.draw_line(close_x + 6.0, close_y - 6.0, close_x - 6.0, close_y + 6.0, close_col, 2.0);
        
        // Render content
        if let Some(c) = &self.content {
            let content_rect = Rect::new(rect.x + 16.0, rect.y + 64.0, rect.width - 32.0, rect.height - 80.0);
            c.render(renderer, content_rect);
        }
        
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        if !self.open {
            return Size { width: 0.0, height: proposal.height.unwrap_or(0.0) };
        }
        let width = 320.0;
        let height = proposal.height.unwrap_or(800.0); // take full height if possible
        
        if let Some(c) = &self.content {
            // we could measure content, but drawer usually has fixed width and takes available height
            let _ = c.intrinsic_size(renderer, SizeProposal::width(width - 32.0));
        }
        
        Size { width, height }
    }
}
