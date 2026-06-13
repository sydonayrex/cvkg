# Modern UI Component System — Required Components

> A synthesized reference list of components that every modern UI system must include, derived from reviewing leading component libraries and design frameworks: Untitled UI, PhiaUI, Rails Blocks, Sencha Ext JS, Studio Graphene, and industry analysis from Codeburst.

---

## How to Read This Document

Components are grouped by functional category, from foundational primitives up to complex, data-heavy, and emerging patterns. Each entry notes why the component is considered standard or increasingly expected. Components marked **↑ Gaining Importance** are those newly elevated to must-have status by modern usage patterns.

---

## 1. Foundation & Design Tokens

These are not components per se, but they underpin every component in the system and must be defined before any UI work begins.

| Token / Element | Purpose |
|---|---|
| **Color palette** (light + dark) | Brand colors, semantic colors (success, error, warning, info), neutral scale |
| **Typography scale** | Font families, sizes, weights, line heights — display, heading, body, code, caption |
| **Spacing & grid system** | 4px or 8px base grid, column grids, gutters, container max-widths |
| **Shadow & elevation** | Drop shadows, inner shadows, blurs — used to communicate depth hierarchy |
| **Border radius tokens** | Consistent rounding across components |
| **Icon library** | A coherent, scalable icon set (SVG-based) |
| **Motion & animation tokens** | Duration, easing curves for transitions — spring, ease-in-out, etc. |

---

## 2. Core Input Components

The atomic building blocks of any form or interactive surface. Every UI system must have these with full variant coverage (default, hover, focus, error, disabled) and WCAG-compliant labeling.

- **Button** — Primary, secondary, ghost, destructive, icon-only, loading state, size variants (xs → xl)
- **Button Group** — Segmented and joined button sets
- **Text Input / Input Field** — With addon support (prefix icon, suffix icon, unit labels), character count, clearable
- **Textarea** — Auto-grow variant, character counter
- **Select (Dropdown)** — Single-value native and custom implementations
- **Combobox / Autocomplete** — Searchable select with async data support
- **Multi-Select** — Tag-style selection of multiple values
- **Checkbox** — With indeterminate state, checkbox groups, select-all pattern
- **Radio Group** — Horizontal and vertical orientations
- **Toggle / Switch** — On/off control with labeled states
- **Slider / Range** — Single and dual-handle (range) variants
- **Rating** — Star or custom-icon rating input
- **Color Picker** — Hex, RGB, HSL input with swatch palette
- **Date Picker** — Single date, date range, date + time, presets
- **Time Picker** — Hour/minute/second selection, 12h/24h modes
- **File Upload / Drop Zone** — Click-to-browse and drag-and-drop, with progress indicator
- **OTP / PIN Input** — Split-field one-time-password entry
- **Password Input** — With show/hide toggle, strength meter
- **Phone Input** — Country code selector + formatted number field
- **Tags Input** — Free-form or restricted tag entry
- **Mention Input** — `@user` and `#topic` autocomplete inside text fields ↑ Gaining Importance
- **Search Input** — With clear button, inline and global variants
- **Editable (Inline Edit)** — Click-to-edit text fields without a separate form ↑ Gaining Importance

---

## 3. Display & Content Components

Components that present information, structure content, and create visual hierarchy.

- **Avatar** — Circular user image with fallback initials, status ring, size variants
- **Avatar Group** — Stacked overlapping avatars with overflow count
- **Badge** — Notification dot, count badge, status badge — in multiple colors and sizes
- **Tag / Chip** — Dismissible or read-only content labels
- **Tooltip** — On-hover/focus contextual hint, with configurable placement
- **Kbd / Hotkey** — Keyboard shortcut display element (e.g. `⌘ K`)
- **Separator / Divider** — Horizontal and vertical rule, with optional label
- **Skeleton Loader** — Placeholder shapes while content loads ↑ Gaining Importance
- **Empty State** — Illustration + message + CTA for zero-data views
- **Accordion** — Collapsible content panels, single and multi-open modes
- **Tabs (Horizontal)** — Switchable content panels, underline and pill variants
- **Tabs (Vertical)** — Sidebar-style tab navigation
- **Card** — Content container with header, body, footer, image variants
- **Timeline / Activity Feed** — Chronological event list ↑ Gaining Importance
- **Code Snippet** — Syntax-highlighted code block with copy button
- **QR Code** — Inline QR code generator ↑ Gaining Importance
- **Typography System** — Heading, body, label, caption, blockquote, list — as styled elements
- **Scroll Area** — Custom-styled scrollable container with visible scrollbars
- **Direction / Locale Toggle** — LTR / RTL layout switching support

---

## 4. Feedback & Overlay Components

These communicate state, prompt action, and layer above the main UI without full-page interruptions.

- **Alert / Inline Alert** — Success, warning, error, info — with icon and dismiss
- **Toast / Snackbar** — Ephemeral notification with action button and auto-dismiss
- **Banner** — Persistent sitewide or section-level notification strip
- **Modal / Dialog** — Focused task or confirmation in an overlay
- **Alert Dialog** — Destructive-action confirmation (cannot be dismissed by clicking outside)
- **Drawer** — Side-panel overlay, drag-to-dismiss on mobile ↑ Gaining Importance
- **Slideover / Sheet** — Similar to drawer; content-heavy side panel
- **Popover** — Rich interactive content bubble anchored to a trigger
- **Hover Card** — Contextual preview card on hover ↑ Gaining Importance
- **Context Menu** — Right-click / long-press contextual action menu ↑ Gaining Importance
- **Dropdown Menu** — Triggered action menu with keyboard navigation
- **Command Palette (⌘K)** — Global keyboard-triggered search/action launcher ↑ Gaining Importance
- **Spinner / Loading Indicator** — Inline and full-page loading states
- **Progress Bar** — Determinate and indeterminate linear progress
- **Circular Progress** — Radial progress ring
- **Skeleton** — (repeated here for visibility as a feedback mechanism)
- **Status Indicator / Dot** — Online, offline, busy, idle — small colored dot
- **Loading Overlay** — Full-component or full-page blocking loader
- **Error Display / Result State** — Structured error + recovery action ↑ Gaining Importance
- **Feedback Widget** — In-product thumbs-up/down or star feedback prompt ↑ Gaining Importance
- **Popconfirm** — Lightweight inline confirmation for irreversible actions ↑ Gaining Importance
- **Dark Mode Toggle** — First-class light/dark/system theme switcher ↑ Gaining Importance

---

## 5. Navigation Components

Components that orient users within a product and allow movement between sections.

- **Navbar / Header Navigation** — Top-bar with logo, links, CTA, and user menu
- **Sidebar / Vertical Navigation** — Collapsible side nav with section groups and icons
- **Mega Menu** — Multi-column dropdown navigation for large link sets ↑ Gaining Importance
- **Breadcrumb** — Hierarchical location trail with truncation for deep paths
- **Pagination** — Page-number and previous/next controls for list data
- **Stepper / Step Tracker** — Multi-step flow progress indicator (linear and non-linear)
- **Bottom Navigation** — Mobile tab bar pinned to the viewport bottom ↑ Gaining Importance
- **Dock Menu** — macOS-style icon dock, desktop and mobile-adaptive ↑ Gaining Importance
- **Floating Nav / Speed Dial** — FAB-style floating action button with expanded options ↑ Gaining Importance
- **Nav Rail** — Compact vertical icon-only navigation for dense layouts ↑ Gaining Importance
- **Toolbar** — Grouped action buttons, often context-sensitive
- **Tree View** — Hierarchical folder/item tree with expand/collapse
- **Table of Contents (ToC)** — Auto-generated anchor-linked document outline ↑ Gaining Importance

---

## 6. Data Display & Table Components

Required for any product with lists, records, or analytical data.

- **Table (Basic)** — Sortable columns, row hover, fixed header
- **Data Table / Data Grid** — Sortable, filterable, paginated, with column resizing and bulk actions
- **Expandable Table** — Rows that expand to show nested detail ↑ Gaining Importance
- **Inline Edit Table** — Cell-level editing without a separate form ↑ Gaining Importance
- **Responsive Table** — Collapses gracefully to card-based layout on small screens ↑ Gaining Importance
- **Pivot Table / Grid** — Summary view of multi-dimensional data (outline, compact, tabular layouts) ↑ Gaining Importance
- **Filter Bar** — Persistent or collapsible filter controls above a data list
- **Bulk Action Bar** — Appears when rows are selected; provides batch operations ↑ Gaining Importance
- **Kanban Board** — Drag-and-drop column board for status-based workflows ↑ Gaining Importance
- **Tree Grid / Tree Table** — Hierarchical data in a table structure

---

## 7. Data Visualization & Charts

Charts are now a standard expectation in dashboards, analytics, and reporting products.

- **Line Chart** — Time-series and trend data
- **Area Chart** — Filled line chart for volume over time
- **Bar / Column Chart** — Comparative category data
- **Stacked Bar/Column Chart** — Part-to-whole composition
- **Pie Chart** — Simple proportion display
- **Donut Chart** — Pie with center KPI label
- **Scatter Chart** — Correlation between two variables
- **Heatmap Chart** — Density or intensity over a grid ↑ Gaining Importance
- **Radar / Spider Chart** — Multi-axis comparison
- **Funnel Chart** — Conversion or pipeline stages
- **Gauge / Dial** — Single KPI displayed on a radial scale
- **Treemap** — Hierarchical proportional rectangles ↑ Gaining Importance
- **Sparkline** — Inline micro-chart within a stat card or table cell ↑ Gaining Importance
- **Stat Card / Metric Card** — KPI number with delta and trend indicator
- **Uptime Bar / Tracker** — Service availability over time ↑ Gaining Importance
- **Bar List** — Ranked horizontal bars in a list ↑ Gaining Importance
- **Leaderboard** — Ranked rows with avatars and scores ↑ Gaining Importance
- **Waterfall Chart** — Cumulative effect of sequential values ↑ Gaining Importance

---

## 8. Calendar & Scheduling Components

Once optional, now standard in any product touching time, booking, or task scheduling.

- **Date Picker** — (see Inputs; repeated here for prominence)
- **Calendar (Monthly View)** — Full-month grid with event markers
- **Calendar (Week View)** — Hour-by-hour weekly schedule ↑ Gaining Importance
- **Calendar (Daily Agenda View)** — Single-day detailed schedule ↑ Gaining Importance
- **Event / Big Calendar** — Full scheduling product — multiple views with drag-and-drop events ↑ Gaining Importance
- **Booking Calendar** — Availability-based slot selection ↑ Gaining Importance
- **Range Calendar / Date Range Picker** — Multi-date span selection with presets
- **Heatmap Calendar** — GitHub-contribution-style activity grid ↑ Gaining Importance
- **Streak Calendar** — Habit tracking / daily completion indicator ↑ Gaining Importance
- **Countdown Timer** — Real-time countdown to a target date/time ↑ Gaining Importance
- **Date Field** — Segmented day/month/year input (accessible, no popup)

---

## 9. Media & File Components

- **Image Upload** — Preview, crop, replace, remove
- **Avatar Upload** — Circular image upload with preview
- **Document Upload** — PDF and file upload with progress
- **Video Player** — Controls, captions, fullscreen ↑ Gaining Importance
- **Audio Player** — Play, pause, scrub, volume ↑ Gaining Importance
- **Lightbox** — Image gallery with zoom overlay
- **Image Comparison** — Before/after slider ↑ Gaining Importance
- **Carousel** — Auto-play or manual image/content slider
- **Aspect Ratio Container** — Locks proportions for responsive media embeds

---

## 10. Layout & Structural Components

Infrastructure components that control spacing, composition, and responsiveness.

- **Container** — Max-width centered wrapper
- **Grid** — Responsive CSS grid layout helper
- **Stack / Flex** — One-directional flex layout utility
- **Divider** — Visual section separator (with optional label)
- **Page Header** — Title, breadcrumb, subtitle, action slot for top of content areas
- **Section Header / Card Header** — Sub-section title with metadata and actions
- **Section Footer** — Pagination or summary below a data section
- **Masonry Grid** — Variable-height card layout ↑ Gaining Importance
- **Split Layout** — Two-pane resizable layout (e.g., list + detail) ↑ Gaining Importance
- **Description List** — Label–value pairs, used in detail/profile views
- **Resizable Panels** — User-adjustable split panes ↑ Gaining Importance
- **Sticky / Fixed Bar** — Header or footer that pins to viewport while scrolling

---

## 11. Specialized Application Patterns

Full UI patterns that combine multiple components into reusable high-level blocks.

- **Rich Text Editor** — WYSIWYG text editing (headings, lists, links, images) ↑ Gaining Importance
- **Messaging / Chat** — Message bubbles, typing indicator, timestamps, reactions ↑ Gaining Importance
- **Notification Center / Feed** — Grouped, filterable notification inbox ↑ Gaining Importance
- **File Manager** — Grid/list browser with folder tree, drag-and-drop, upload ↑ Gaining Importance
- **Settings Page** — Tabbed or sidebar-navigated settings with section groups
- **Profile Page** — User card, stats, activity feed layout
- **Dashboard Layout** — Sidebar + header + main content region with widget grid
- **Point-of-Sale (POS) Layout** — Catalog + cart + checkout flow ↑ Gaining Importance
- **Pricing Page / Cards** — Plan comparison, feature lists, CTA
- **Two-Factor / OTP Verification** — Code entry with resend flow

---

## 12. AI & Emerging Components ↑ Gaining Importance

A new category that is rapidly becoming expected in modern products.

- **AI Chat Interface** — Conversational input with model output streaming and typing indicator
- **AI Chat (Advanced)** — Model selector, temperature control, conversation history panel
- **Image Generation Studio** — Prompt editor, style picker, result gallery
- **Command Palette with AI** — Natural language action search powered by AI
- **Suggestion Chips** — Pre-written prompt suggestions in AI/chat interfaces
- **Watermark** — Brand overlay for generated or protected media

---

## 13. Animation & Visual Effect Utilities ↑ Gaining Importance

Once considered decorative, these are now standard for communicating state and creating premium-feel interfaces.

- **Typewriter / Text Scramble Effect** — Animated character-by-character text reveal
- **Number Ticker** — Animated count-up for metrics and dashboards
- **Shimmer / Skeleton Shimmer** — Motion variant of skeleton loaders
- **Marquee** — Horizontally scrolling content strip (logos, testimonials)
- **Animated Border / Beam** — Glowing border motion effects for cards
- **Confetti Burst** — Success celebration animation
- **Fade / Float Entrances** — Scroll-triggered reveal animations
- **Spotlight / Card Spotlight** — Mouse-following highlight on cards
- **Gradient Mesh Background** — Fluid color gradient canvas backgrounds
- **Particle / Aurora Background** — Atmospheric animated backgrounds for hero sections

---

## 14. Cross-Cutting Requirements (Non-Negotiable Qualities)

Every component in the system must satisfy these baseline requirements — they are not optional extras.

| Requirement | Standard |
|---|---|
| **Accessibility (WCAG 2.1 AA)** | ARIA roles, keyboard navigation, focus management, screen reader support |
| **Dark Mode** | CSS custom properties / Tailwind dark class — all components must support both themes |
| **Responsive Design** | All components adapt from 320px (mobile) to 1920px+ (desktop) without breaking |
| **TypeScript Support** | Typed props, exported interfaces, IDE autocompletion |
| **Empty & Error States** | Every data-displaying component must handle zero-data and fetch-error gracefully |
| **Loading States** | Skeleton or spinner while data is in flight |
| **Internationalization (i18n)** | RTL layout support, locale-aware date/number formatting |
| **Motion Sensitivity** | Respect `prefers-reduced-motion` — disable or reduce animations when set |
| **Performance** | Virtualized lists/grids for large datasets; no layout shift on load |
| **Theming & Tokens** | All visual values derived from design tokens, not hardcoded |

---

## Summary Count

| Category | Approximate Component Count |
|---|---|
| Foundation & Design Tokens | 7 token groups |
| Core Input Components | 22 |
| Display & Content | 21 |
| Feedback & Overlay | 22 |
| Navigation | 13 |
| Data Display & Tables | 10 |
| Data Visualization & Charts | 17 |
| Calendar & Scheduling | 11 |
| Media & File | 9 |
| Layout & Structural | 13 |
| Specialized Application Patterns | 10 |
| AI & Emerging Components | 6 |
| Animation & Visual Effects | 10 |
| **Total** | **~171 components / component types** |

---

*Compiled June 2026 from: Untitled UI (React component library), PhiaUI (Phoenix LiveView), Rails Blocks, Sencha Ext JS blog, Studio Graphene UI/UX blog, and Codeburst design system analysis.*
