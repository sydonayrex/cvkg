# UI Component Library Review
> Consolidated review of components across 13 sources — deduplicated by category and intent.
> Reviewed: June 12, 2026

---

## Sources Reviewed

| # | Source | Focus |
|---|--------|-------|
| 1 | [Agent Elements (21st.dev)](https://agent-elements.21st.dev/docs/agent-chat) | AI agent chat UI primitives |
| 2 | [Aceternity UI](https://ui.aceternity.com/components) | Animated marketing & hero components |
| 3 | [assistant-ui](https://github.com/assistant-ui/assistant-ui) | React/TypeScript AI chat library |
| 4 | [stackzero-labs/ui (Commerce UI)](https://github.com/stackzero-labs/ui) | E-commerce components |
| 5 | [S5SAJID/custom-ecom](https://github.com/S5SAJID/custom-ecom) | E-commerce admin dashboard template |
| 6 | [olliethedev/dnd-dashboard](https://github.com/olliethedev/dnd-dashboard) | Drag-and-drop dashboard |
| 7 | [birobirobiro/awesome-shadcn-ui](https://github.com/birobirobiro/awesome-shadcn-ui) | Curated shadcn/ui ecosystem directory |
| 8 | [AgusMayol/optics](https://github.com/agusmayol/optics) | 60+ accessible Base UI component library |
| 9 | [brijr/components](https://github.com/brijr/components) | Next.js marketing page section components |
| 10 | [BadtzUI](https://www.badtz-ui.com/docs/components/3d-wrapper) | Animated & 3D effect components |
| 11 | [uselayouts](https://uselayouts.com/docs/components/3d-book) | Animated Framer Motion micro-interaction components |
| 12 | [aliimam.in](https://aliimam.in/docs/components) | Backgrounds, patterns & specialized components |
| 13 | [pqoqubbw/icons (lucide-animated)](https://github.com/pqoqubbw/icons) | Animated icon library |

---

## Consolidated Component Categories

Components are grouped by function. Where multiple sources offer the same type of component, they are listed together with source attribution.

---

### 1. AI Chat Shell / Conversation Container
Full chat interface that combines message display, input, and streaming state management.

| Component Name | Source | Intent |
|----------------|--------|--------|
| AgentChat | Agent Elements | Complete chat surface combining message list, input bar, tool output, error handling, and streaming. Drop-in for agentic UIs. |
| Thread / AssistantUI | assistant-ui | Composable primitive-based chat thread with full AI SDK, LangGraph, and Mastra integration. Handles streaming, auto-scroll, retries, and keyboard accessibility. |

---

### 2. Message Display
Components that render individual messages or a scrollable list of messages.

| Component Name | Source | Intent |
|----------------|--------|--------|
| MessageList | Agent Elements | Scrollable, auto-scrolling list of chat messages supporting tool outputs, markdown, image previews, and copy toolbar. |
| UserMessage | Agent Elements | Styled bubble for user-authored messages. |
| Markdown | Agent Elements | Renders AI markdown responses with code highlighting inside a message. |
| ErrorMessage | Agent Elements | Displays inline error state within the message stream. |

---

### 3. Chat Input
Input bar and related controls for composing and sending messages.

| Component Name | Source | Intent |
|----------------|--------|--------|
| InputBar | Agent Elements | Expandable text input with file attachment, paste-to-attach, drag-over state, question-bar override, and send/stop control. |
| SendButton | Agent Elements | Animated send/stop toggle button wired to streaming status. |
| AttachmentButton | Agent Elements | Opens file/image picker and manages attached items before sending. |
| FileAttachment | Agent Elements | Renders an attached file pill with name, size, and remove action. |
| Suggestions | Agent Elements | Horizontally scrollable chip row of prompt shortcuts shown when input is empty. |
| Gooey Input | Aceternity UI | Search-style input that expands with a gooey SVG filter animation on focus. |
| Placeholders & Vanish Input | Aceternity UI | Input with animated rotating placeholder text that vanishes on submit. |
| Morphing Input | uselayouts | Animated input that morphs shape/size based on interaction state. |

---

### 4. Model & Mode Selectors
Controls for switching AI model or chat mode.

| Component Name | Source | Intent |
|----------------|--------|--------|
| ModelPicker | Agent Elements | Dropdown for selecting which AI model to use in the current session. |
| ModeSelector | Agent Elements | Segmented control for switching between agent operating modes (e.g., chat vs. code). |

---

### 5. AI Tool Call Cards
Discrete components that visualize a specific AI tool execution inline in the message thread.

| Component Name | Source | Intent |
|----------------|--------|--------|
| BashTool | Agent Elements | Shows shell command execution with stdout/stderr output. |
| EditTool | Agent Elements | Displays file edit diffs with before/after view. |
| SearchTool | Agent Elements | Renders web search queries and result summaries. |
| TodoTool | Agent Elements | Shows a todo/task list being managed by the agent. |
| PlanTool | Agent Elements | Displays a multi-step plan with progress tracking. |
| ThinkingTool | Agent Elements | Shows the agent's reasoning/thinking process inline. |
| QuestionTool | Agent Elements | Renders an interactive clarifying question the agent is asking the user; can be multi-step. |
| McpTool | Agent Elements | Generic display for MCP-protocol tool calls. |
| GenericTool | Agent Elements | Catch-all card for any unrecognized tool type. |
| SubagentTool | Agent Elements | Shows a spawned sub-agent task with its own status. |
| ToolGroup | Agent Elements | Visually groups multiple related tool calls together. |

---

### 6. Loading & Streaming Indicators
Visual feedback for async operations and streaming responses.

| Component Name | Source | Intent |
|----------------|--------|--------|
| TextShimmer | Agent Elements | Shimmer animation over text to indicate streaming in progress. |
| SpiralLoader | Agent Elements | Spiral SVG animation used as a thinking/loading indicator in agent UIs. |
| Loaders | Aceternity UI | Set of minimal spinners for loading screens and components. |
| Multi-Step Loader | Aceternity UI | Step-through progress loader for long-running operations (e.g., app startup). |
| Status Button | uselayouts | Button that cycles through idle → loading → success/error states with animation. |

---

### 7. Buttons
Interactive trigger elements with varying feedback behaviors.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Stateful Button | Aceternity UI | Button that shows loading, then success state after an async action. |
| Magnetic Button | Aceternity UI | Button that drifts toward the cursor on hover using spring physics. |
| Moving Border | Aceternity UI | Animated gradient border orbiting a button or card to draw attention. |
| Delete Button | uselayouts | Confirms destructive actions with a swipe-to-confirm micro-interaction. |
| Discover Button | uselayouts | Animated CTA button with reveal effect. |
| Glowing Button | BadtzUI | Button with an animated radial glow effect on hover. |
| Swipe Button | BadtzUI | Slider-style button requiring a swipe gesture to confirm. |
| Gradient Slide Button | BadtzUI | Button whose gradient fill slides in on hover. |
| Star Button | BadtzUI | Button that emits star particles on click. |
| Confetti Button | BadtzUI | Triggers a confetti burst on activation. |
| Shuffle Button | BadtzUI | Button that visually shuffles its label characters on hover. |
| Stagger Button | BadtzUI | Characters stagger-animate in/out on hover. |
| Like Button | BadtzUI | Toggle favorite/like with animated heart feedback. |

---

### 8. Cards
Container components with specialized visual behavior or interaction.

| Component Name | Source | Intent |
|----------------|--------|--------|
| 3D Card Effect | Aceternity UI | Card that tilts on mouse movement to create a depth/perspective effect. |
| Wobble Card | Aceternity UI | Card that translates and scales on mouse move; good for feature highlights. |
| Card Spotlight | Aceternity UI | Reveals a radial gradient spotlight under the cursor while hovering a card. |
| Comet Card | Aceternity UI | 3D perspective tilt card as seen on Perplexity Comet. |
| Glare Card | Aceternity UI | Reflective glare effect on hover (as seen on Linear). |
| Evervault Card | Aceternity UI | Scrambling encrypted-text and gradient reveal on hover. |
| Expandable Card | Aceternity UI / BadtzUI | Card that expands on click to show additional content. |
| Draggable Card | Aceternity UI | Card that can be dragged and snaps back to bounds. |
| Focus Cards | Aceternity UI | Gallery of cards where hovering one blurs the rest. |
| Card Stack | Aceternity UI | Cards visually stacked and cycled on interval; good for testimonials. |
| Hover Effect Cards | Aceternity UI | Set of cards with a shared highlight that slides to the hovered card. |
| Glowing Background Stars Card | Aceternity UI | Card with animated star particles in the background. |
| Cursor Cards | BadtzUI | Cards that subtly track and react to cursor position. |
| Flipping Card | BadtzUI | Card with a 3D flip reveal (front/back faces). |
| Animated Cards 1–3 | BadtzUI | Three distinct decorative card animation styles for marketing layouts. |
| Shake Testimonial Card | uselayouts | Testimonial card that shakes to draw attention on hover. |
| Bento Card | uselayouts | Bento-grid-compatible card with configurable slot areas. |
| Pricing Card | uselayouts | Self-contained pricing tier card with toggle and feature list. |

---

### 9. 3D & Spatial Components
Components that use CSS 3D transforms or WebGL for immersive effects.

| Component Name | Source | Intent |
|----------------|--------|--------|
| 3D Wrapper | BadtzUI | Wraps any content and applies cursor-tracked 3D rotation and perspective depth. |
| 3D Book | uselayouts | Interactive 3D book that opens/closes on hover or drag using CSS 3D transforms. |
| 3D Globe | Aceternity UI | Realistic rotating globe with tooltips and avatar pins. |
| GitHub Globe | Aceternity UI | Interactive globe styled for contribution activity visualization. |
| World Map | Aceternity UI | SVG world map with animated connection lines between locations. |
| 3D Marquee | Aceternity UI | Marquee displayed on a 3D grid plane; works well for testimonials/screenshots. |
| 3D Animated Pin | Aceternity UI | Gradient pin that lifts and animates on hover; good for product links on a map. |
| Container Scroll Animation | Aceternity UI | Hero element that rotates in 3D as the user scrolls. |
| Macbook Scroll | Aceternity UI | Image appears to emerge from a MacBook lid as the user scrolls. |
| Device Frame | aliimam.in | Responsive device mockup (MacBook, iMac, iPhone, iPad) for showcasing UI screenshots. |

---

### 10. Backgrounds & Visual Effects
Full-surface or section-level visual treatments.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Aurora Background | Aceternity UI | Soft aurora borealis gradient animation for hero backgrounds. |
| Background Beams | Aceternity UI | Animated SVG beams following a path; good for dark hero sections. |
| Background Beams with Collision | Aceternity UI | Exploding beams that collide and scatter. |
| Background Gradient Animation | Aceternity UI | Smoothly shifting gradient that moves over time. |
| Background Ripple Effect | Aceternity UI | Grid of cells that ripple outward when the user clicks. |
| Background Lines | Aceternity UI | Animated SVG wave paths across the background. |
| Grid & Dot Backgrounds | Aceternity UI | Simple CSS grid or dot pattern backgrounds. |
| Vortex Background | Aceternity UI | Swirling vortex background good for CTAs. |
| Wavy Background | Aceternity UI | Animated sine-wave canvas background. |
| Shooting Stars Background | Aceternity UI | Stars and shooting-star animation layer. |
| Meteor Effect | Aceternity UI | Diagonal beam streaks across a card or section. |
| Noise Background | Aceternity UI | Dynamic gradient with noise texture overlay. |
| Dotted Glow Background | Aceternity UI | Dot pattern with pulsing glow effect. |
| Scales | Aceternity UI | Repeating diagonal/horizontal/vertical line pattern. |
| Dither Shader | Aceternity UI | Real-time ordered dithering (pixel art / retro aesthetic) applied to images. |
| Pixel Distortion Shader | BadtzUI | WebGL fragment shader for per-pixel distortion effects. |
| Pulse Shader | BadtzUI | Pulsing radial shader effect. |
| Mouse Wave Shader | BadtzUI | Shader that generates wave distortion following the mouse. |
| Stripe Animated Gradient | BadtzUI | Stripe-style animated diagonal gradient background. |
| Hyperspace Background | BadtzUI | Star-warp / hyperspace travel animation background. |
| Pixelated Canvas | Aceternity UI | Converts an image to a pixelated canvas with mouse-distortion interaction. |
| Particles | BadtzUI | Configurable floating particles system for background depth. |
| Canvas Reveal Effect | Aceternity UI | Dot background that expands on hover (as seen on Clerk). |
| Background Boxes | Aceternity UI | Full-width grid of boxes that highlight on hover. |

---

### 11. Text Effects & Typography
Animated or interactive text rendering.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Text Flipping Board | Aceternity UI | Split-flap display that animates characters with a mechanical flip transition. |
| Typewriter Effect | Aceternity UI | Characters type themselves onto the screen sequentially. |
| Text Generate Effect | Aceternity UI | Text fades in word-by-word on load. |
| Flip Words | Aceternity UI | Cycles through a list of words with a vertical flip animation. |
| Container Text Flip | Aceternity UI | A container that flips between multiple words, animating its own width. |
| Layout Text Flip | Aceternity UI | Text flip that shifts surrounding layout elements during transition. |
| Encrypted Text | Aceternity UI | Text that gradually resolves from random characters to its final value. |
| Colourful Text | Aceternity UI | Text with per-character color, filter, and scale effects. |
| Text Hover Effect | Aceternity UI | Gradient outline animates across text on hover (as seen on x.ai). |
| Squiggly Text | Aceternity UI | SVG turbulence displacement filter gives text a hand-drawn squiggle. |
| Canvas Text | Aceternity UI | Colorful curved lines rendered on a canvas, clipped to text shape. |
| Pointer Highlight | Aceternity UI | Highlights text in view with a pointer animation. |
| Blur Reveal | BadtzUI | Text blurs in from invisible to sharp on scroll or mount. |
| Fade Up Word | BadtzUI | Words translate and fade upward sequentially. |
| Stagger Blur Effect | BadtzUI | Characters stagger-reveal with blur-to-sharp animation. |

---

### 12. Navigation & Menus
Header, navbar, sidebar, and dock navigation patterns.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Resizable Navbar | Aceternity UI | Navbar that changes width on scroll and shrinks when scrolling down. |
| Floating Navbar | Aceternity UI | Sticky navbar that hides on scroll down, re-appears on scroll up. |
| Navbar Menu | Aceternity UI | Mega-nav style menu with animated hover highlight between items. |
| Floating Dock | Aceternity UI | macOS dock-style navigation bar with magnification on hover. |
| Sidebar | Aceternity UI | Expandable sidebar that expands on hover; mobile-responsive. |
| Notch | Aceternity UI | Floating configurable notch pinned to the top or bottom of the screen; animates active state. |
| Sticky Banner | Aceternity UI | Top-of-page announcement banner that hides on scroll down. |
| Bottom Menu | uselayouts | Mobile-first bottom navigation bar with animated active indicator. |
| Dynamic Toolbar | uselayouts | Contextual toolbar that appears and morphs based on current selection or state. |
| Discrete Tabs | uselayouts | Tabs with a subtle underline sliding indicator. |
| Vertical Tabs | uselayouts | Sidebar-oriented tab list with animated active state. |
| Smooth Dropdown | uselayouts | Dropdown menu with spring-physics open/close animation. |
| Dock | BadtzUI | Another macOS-style icon dock with hover magnification. |
| Animated Tabs | Aceternity UI | Tabs with animated background highlight sliding between items. |

---

### 13. Modals, Drawers & Overlays
Dialogs, sheets, and overlay patterns.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Animated Modal | Aceternity UI | Composable modal with entrance/exit transitions. |
| Link Preview | Aceternity UI | Popover that shows a live preview of a linked URL on hover. |
| Animated Tooltip | Aceternity UI | Tooltip that follows the mouse with spring animation and image support. |
| Tooltip Card | Aceternity UI | Card-sized tooltip container that tracks the mouse pointer. |

---

### 14. Carousels, Galleries & Media
Scrolling, sliding, or expanding image/content display.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Apple Cards Carousel | Aceternity UI | Minimal swipeable card carousel (as seen on Apple.com). |
| Carousel | Aceternity UI | Customizable general-purpose carousel with micro-interactions. |
| Infinite Moving Cards | Aceternity UI | Cards scroll in an infinite loop (marquee-style); good for testimonials. |
| Images Slider | Aceternity UI | Full-page image slider with keyboard navigation. |
| Parallax Grid Scroll | Aceternity UI | Two-column grid where columns scroll in opposite directions. |
| Hero Parallax | Aceternity UI | Scroll-driven rotation, translation, and opacity animations for hero images. |
| Compare | Aceternity UI | Side-by-side image comparison with a draggable divider. |
| Layout Grid | Aceternity UI | Grid that animates selected item to full size using Framer Motion layout. |
| Lens | Aceternity UI | Zoom lens overlay for images, videos, or any content. |
| Image Split | BadtzUI | Image that splits apart to reveal content underneath. |
| Image Trail | BadtzUI | A trail of images follows the cursor across a container. |
| Expandable Gallery | uselayouts | Image gallery that expands selected items inline. |
| Feature Carousel | uselayouts | Carousel designed for showcasing feature highlights with text and media. |
| Fluid Expanding Grid | uselayouts | Grid where clicking an item expands it fluidly within the layout. |
| Parallax Hero Images | Aceternity UI | Mouse-driven parallax on layered hero images at different depths. |
| ASCII Art | Aceternity UI | Converts any image to ASCII art with configurable charsets and animation. |
| Webcam Pixel Grid | Aceternity UI | Live webcam feed rendered as a pixel grid. |

---

### 15. Scroll Animations & Reveal
Components activated or driven by the user's scroll position.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Tracing Beam | Aceternity UI | Animated beam that traces an SVG path as the user scrolls. |
| Sticky Scroll Reveal | Aceternity UI | Content sticks while scrolling; each section reveals on scroll. |
| Timeline | Aceternity UI | Vertical timeline with sticky header and scroll-following beam. |
| Hero Highlight | Aceternity UI | Background effect with text highlight that activates on scroll into view. |
| Google Gemini Effect | Aceternity UI | SVG animation replicating the Gemini logo reveal on scroll. |
| SVG Mask Effect | Aceternity UI | Masks reveal underlying content as the cursor moves over a container. |
| Direction Aware Hover | Aceternity UI | Card hover effect that knows which edge the cursor entered from. |

---

### 16. Testimonials & Social Proof
Pre-built patterns for displaying user reviews and credibility signals.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Animated Testimonials | Aceternity UI | Minimal testimonial component with image and quote. |
| Social Proof Avatars | BadtzUI | Overlapping avatar stack with a count badge for social proof. |
| Images Badge | Aceternity UI | Badge with stacked avatar images that expand on hover. |
| Empty Testimonial | uselayouts | Placeholder state for a testimonial section before content is added. |

---

### 17. Forms & Inputs
Form-level or field-level input components.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Signup Form | Aceternity UI | Pre-styled sign-up form built on shadcn inputs with Framer Motion. |
| File Upload | Aceternity UI | Drag-and-drop file upload with background grid and micro-interactions. |
| Multi-Step Form | uselayouts | Wizard-style form with animated step transitions. |
| Inline Edit | uselayouts | Click-to-edit field that toggles between display and edit mode inline. |
| Day Picker | uselayouts | Compact calendar date picker component. |

---

### 18. Data Display & Dashboards
Charts, stats, tables, and dashboard-specific widgets.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Bento Grid | Aceternity UI | Skewed grid layout for showcasing features with varied card sizes. |
| DnD Dashboard | dnd-dashboard | Full drag-and-drop dashboard where panels can be rearranged via drop-to-swap using swapy. |
| Stacked List | uselayouts | Vertically stacked list with depth/shadow effect suggesting layers. |
| Filter Interaction | uselayouts | Animated filter/sort control for lists with smooth reflow. |
| Folder Interaction | uselayouts | File-system-like folder that opens/closes to show nested content. |
| Bucket | uselayouts | Drag-and-drop bucket component for collecting items. |
| Gauge / Pixelated Grid | aliimam.in | Gauge dial and pixelated grid data visualization components. |

---

### 19. Marquees & Tickers
Continuously scrolling content strips.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Marquee | BadtzUI / aliimam.in | Looping horizontal or vertical scroll strip; supports adjustable speed, direction, and pause-on-hover. |
| Infinite Ribbon | BadtzUI | Continuous ribbon scroll for logos, text, or imagery. |
| Cloud Orbit | BadtzUI | Icons or images orbiting in a circular loop animation. |

---

### 20. Icons
Icon systems and animated icon libraries.

| Component Name | Source | Intent |
|----------------|--------|--------|
| lucide-animated (pqoqubbw/icons) | pqoqubbw/icons | Beautifully crafted animated versions of Lucide icons. Each icon animates on hover or trigger; available as React components with tree-shaking. |
| Animated Keyboard | BadtzUI | A keyboard component with mechanical key animations. |

---

### 21. E-Commerce Components
Product, cart, review, and storefront-specific components.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Rating Star | Commerce UI (stackzero) | Star rating display for products; supports half-star and review counts. |
| Product Card | Commerce UI (stackzero) | Card for displaying a product with image, title, price, and add-to-cart. |
| Product Detail / Specs | Commerce UI (stackzero) | Detailed product view with feature list, availability badge, and buy actions. |
| Review Card | Commerce UI (stackzero) | Displays a customer review with rating, avatar, and date. |
| Review Summary | Commerce UI (stackzero) | Aggregated rating breakdown (5-star distribution bar chart style). |
| Flash Sale / Countdown Banner | Commerce UI (stackzero) | Promotional banner with a live countdown timer for time-limited offers. |
| New In Stock Badge | Commerce UI (stackzero) | "New in Stock" or "Just Released" annotation overlay for product cards. |
| Admin Dashboard (custom-ecom) | S5SAJID/custom-ecom | Full admin panel template with product management, orders, customers, and settings pages. |

---

### 22. Marketing Page Sections
Pre-built full-section layouts for landing and marketing pages.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Hero Sections (1–6) | brijr/components | Six hero layout variants combining headline, sub-headline, image, and CTA. |
| Feature Sections (1–9) | brijr/components | Nine feature block layouts (text + icon, text + image, card grid) for highlighting product capabilities. |
| CTA Sections (1–4) | brijr/components | Call-to-action blocks with headline, copy, and buttons or email capture. |
| FAQ Sections (1–2) | brijr/components | Accordion-based FAQ sections with two layout variants. |
| Pricing Sections (1–4) | brijr/components | Pricing plan cards with tiered options; variants include monthly/annual toggle and per-seat pricing. |
| Header Sections (1–2) | brijr/components | Page-top header blocks with headline and CTA buttons. |
| Footer Sections (1–5) | brijr/components | Five footer layout variants covering minimal, linked-columns, and newsletter-capture styles. |

---

### 23. General-Purpose UI Primitives & Component Systems
Full component libraries providing broad sets of foundational UI elements.

| Component Name | Source | Intent |
|----------------|--------|--------|
| Optics (60+ components) | AgusMayol/optics | Accessible, Base UI–grounded component library with Tailwind v4 and dark mode. Covers buttons, selects, dialogs, tables, badges, inputs, and more. |
| awesome-shadcn-ui | birobirobiro/awesome-shadcn-ui | Directory of 150+ third-party shadcn/ui-compatible libraries, components, tools, and templates. Not a component library itself — a discovery resource. |

---

### 24. Miscellaneous / Specialty Components

| Component Name | Source | Intent |
|----------------|--------|--------|
| Terminal | Aceternity UI | macOS-style terminal component with bash syntax highlighting and typewriter effect. |
| Keyboard | Aceternity UI | Visual macOS keyboard with mechanical key sound effects on interaction. |
| Code Block | Aceternity UI | Configurable syntax-highlighted code display block. |
| Attraction (Physics) | aliimam.in | Wrapper that applies Matter.js physics to React children, making them attract or repel. |
| Magnified Bento | uselayouts | Bento grid where hovering magnifies the hovered cell. |
| Sparkles | Aceternity UI | Configurable sparkle particle effect layered over content or used as a standalone background. |
| Following Pointer | Aceternity UI | Custom cursor that follows the mouse and animates label content. |
| Spotlight / Spotlight New | Aceternity UI | Cursor-following radial spotlight effect good for dark sections. |
| Lamp Effect | Aceternity UI | Lamp-style backlit glow effect for section headers (as seen on Linear). |

---

## Cross-Source Redundancy Notes

Many component types appear across multiple libraries. The most duplicated are:

- **Marquee/infinite scroll strip** — Agent Elements, Aceternity, BadtzUI, aliimam.in
- **Testimonial cards** — Aceternity, BadtzUI, uselayouts, brijr/components
- **Animated buttons** — BadtzUI, Aceternity, uselayouts (all offer glowing, swipe, gradient-slide variants)
- **Hero sections** — brijr/components, Aceternity blocks, badtz backgrounds
- **Sidebar / dock navigation** — Aceternity, BadtzUI (near-identical macOS dock pattern)
- **Expandable cards** — Aceternity and BadtzUI each have their own version
- **3D card tilt** — Aceternity (3D Card Effect, Wobble Card, Comet Card) and BadtzUI (3D Wrapper, Cursor Cards)
- **Chat / AI interfaces** — Agent Elements and assistant-ui overlap heavily in intent; assistant-ui is lower-level primitives, Agent Elements are higher-level pre-styled components

When picking between duplicates, prefer the library whose styling system (shadcn/Tailwind/Radix) already matches your project.
