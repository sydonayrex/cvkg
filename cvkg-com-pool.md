# CVKG Component Pool -- Inspiration & Recommendations

## Sources Reviewed (20 libraries)
- https://agent-elements.21st.dev/docs/agent-chat
- https://ui.aceternity.com/components
- https://magicui.design/docs/components
- https://www.badtz-ui.com/docs/components/3d-wrapper
- https://www.cult-ui.com/docs/components/hero-color-panels
- https://m3.material.io/components _(added in v3)_
- https://mui.com/x/whats-new/
- https://v7.mui.com/joy-ui/getting-started/
- https://uselayouts.com/docs/components/3d-book
- https://aliimam.in/docs/components
- https://github.com/pqoqubbw/icons
- https://www.kibo-ui.com/components/avatar-stack
- https://tailark.com
- https://github.com/assistant-ui/assistant-ui
- https://github.com/stackzero-labs/ui
- https://github.com/S5SAJID/custom-ecom
- https://github.com/olliethedev/dnd-dashboard
- https://github.com/birobirobiro/awesome-shadcn-ui
- https://github.com/agusmayol/optics
- https://github.com/brijr/components
- https://github.com/assistant-ui/assistant-ui
- https://github.com/stackzero-labs/ui
- https://github.com/S5SAJID/custom-ecom
- https://github.com/olliethedev/dnd-dashboard
- https://github.com/birobirobiro/awesome-shadcn-ui
- https://github.com/agusmayol/optics
- https://github.com/brijr/components
- https://www.badtz-ui.com/docs/components/3d-wrapper
- https://uselayouts.com/docs/components/3d-book
- https://aliimam.in/docs/components
- https://github.com/pqoqubbw/icons
- https://magicui.design/docs/components
- https://www.kibo-ui.com/components/avatar-stack
- https://tailark.com
- https://www.cult-ui.com/docs/components/hero-color-panels _(added in v2)_
- https://mui.com/x/whats-new/ _(added in v2)_
- https://v7.mui.com/joy-ui/getting-started/ _(added in v2)_
- https://github.com/assistant-ui/assistant-ui
- https://github.com/stackzero-labs/ui
- https://github.com/S5SAJID/custom-ecom
- https://github.com/olliethedev/dnd-dashboard
- https://github.com/birobirobiro/awesome-shadcn-ui
- https://github.com/agusmayol/optics
- https://github.com/brijr/components
- https://www.badtz-ui.com/docs/components/3d-wrapper
- https://uselayouts.com/docs/components/3d-book
- https://aliimam.in/docs/components
- https://github.com/pqoqubbw/icons
- https://magicui.design/docs/components
- https://www.kibo-ui.com/components/avatar-stack
- https://tailark.com

---

## 1. AGENT / AI COMPONENTS (from agent-elements.21st.dev + assistant-ui)

### Already Have (improve these):
- `HuginChat` -- basic chat. **Improve**: Add streaming message support, tool call display, markdown rendering, copy-to-clipboard on messages.
- `TokenStream` -- streaming text. **Improve**: Add typing indicator, word-level animation.
- `GeriPrompt` -- prompt input. **Improve**: Add slash commands, file attachments, model selector.

### Missing (implement these):

| Component | Intent | Priority |
|---|---|---|
| **AgentChat** | Full chat surface with messages, status, send/stop handlers. Composable sub-components. | HIGH |
| **MessageList** | Scrollable list of chat messages with auto-scroll, group by role. | HIGH |
| **InputBar** | Text input with send button, stop button, character count. | HIGH |
| **Suggestions** | Quick-action suggestion chips below input. | MEDIUM |
| **Model Picker** | Dropdown to select AI model (GPT-4, Claude, etc.). | MEDIUM |
| **Mode Selector** | Toggle between chat modes (ask, search, agent). | MEDIUM |
| **UserMessage** | Styled user message bubble with avatar. | HIGH |
| **AssistantMessage** | Styled assistant message with avatar, actions (copy, retry, thumbs). | HIGH |
| **Markdown** | Render markdown with code highlighting, tables, links. | HIGH |
| **SendButton** | Animated send/stop button with loading state. | MEDIUM |
| **AttachmentButton** | File attachment trigger with drag-and-drop zone. | MEDIUM |
| **FileAttachment** | Display attached file with name, size, remove button. | MEDIUM |
| **TextShimmer** | Animated shimmer loading placeholder for streaming text. | MEDIUM |
| **SpiralLoader** | Animated spiral loading indicator. | LOW |
| **ToolCard** | Display tool call with name, args, status, result. | HIGH |
| **ToolGroup** | Group multiple tool calls with expand/collapse. | MEDIUM |
| **BashTool** | Terminal-style display for bash tool calls. | MEDIUM |
| **EditTool** | Diff-style display for file edit tool calls. | MEDIUM |
| **SearchTool** | Display search queries and results. | MEDIUM |
| **TodoTool** | Interactive todo list from agent. | MEDIUM |
| **PlanTool** | Display agent plan with steps and progress. | MEDIUM |
| **SubagentTool** | Display subagent delegation and results. | LOW |
| **QuestionTool** | Interactive question/answer from agent. | MEDIUM |
| **McpTool** | Display MCP tool calls and results. | LOW |
| **ThinkingTool** | Collapsible thinking/reasoning display. | MEDIUM |
| **GenericTool** | Fallback display for unknown tool types. | LOW |
| **CopyToolbar** | Floating toolbar with copy, share, export actions. | MEDIUM |

---

## 2. ANIMATED / EFFECT COMPONENTS (from aceternity + magicui + badtz-ui)

### Already Have (improve these):
- `HolographicRunestone` -- holographic effect. **Improve**: Add more shader variants.
- `BifrostTabs` -- glass tabs. **Improve**: Add animated indicator.
- `Effects` -- general effects. **Improve**: Add more effect types from aceternity.
- `Shatter` -- shatter effect. **Improve**: Add physics-based animation.
- `Lightning` -- lightning bolt. **Improve**: Add glow and particle effects.

### Missing (implement these):

#### Text Animations
| Component | Intent | Priority |
|---|---|---|
| **TextAnimate** | Animate text with various effects (fade, slide, scale, blur). | HIGH |
| **TypewriterEffect** | Typewriter-style character-by-character reveal. | HIGH |
| **TextGenerateEffect** | Word-by-word reveal with blur fade-in. | MEDIUM |
| **FlipWords** | Animated word rotation/flipping. | MEDIUM |
| **TextHoverEffect** | Text that responds to hover with distortion. | LOW |
| **HeroHighlight** | Highlighted text with animated underline. | MEDIUM |
| **TextRevealCard** | Card with text that reveals on scroll/hover. | MEDIUM |
| **NumberTicker** | Animated number counter with rolling digits. | HIGH |
| **AnimatedShinyText** | Text with animated shine/shimmer sweep. | MEDIUM |
| **AnimatedGradientText** | Text with animated gradient color shift. | MEDIUM |
| **AuroraText** | Text with aurora borealis effect background. | LOW |
| **SparklesText** | Text with animated sparkle particles. | LOW |
| **MorphingText** | Text that morphs between different strings. | LOW |
| **SpinningText** | Text that spins/rotates on hover. | LOW |
| **LineShadowText** | Text with animated line shadow effect. | LOW |
| **VideoText** | Text with video fill/clip. | LOW |
| **ScrollBasedVelocity** | Text that scales based on scroll velocity. | LOW |
| **WordRotate** | Rotating words in a sentence. | MEDIUM |
| **Text3DFlip** | 3D flip animation for text. | LOW |
| **TextHighlighter** | Animated text highlighter/marker effect. | MEDIUM |
| **HyperText** | Text with hyperlink-style hover animations. | LOW |
| **KineticText** | Physics-based kinetic text animation. | LOW |
| **ComicText** | Comic book-style text with halftone effect. | LOW |
| **EncryptedText** | Text that decrypts/reveals on hover. | LOW |
| **SquigglyText** | Text with squiggly/wavy underline animation. | LOW |
| **LayoutTextFlip** | Layout-aware text flip animation. | LOW |
| **ContainerTextFlip** | Text flip within container bounds. | LOW |
| **TextFlippingBoard** | Flip board style text animation. | LOW |
| **CanvasText** | Text rendered on canvas with effects. | LOW |
| **WisprFlowText** | Text with whisper/flow animation. | LOW |

#### Card Effects
| Component | Intent | Priority |
|---|---|---|
| **CardStack** | Stacked cards with depth and parallax. | HIGH |
| **CardHoverEffect** | Card with 3D tilt on hover. | HIGH |
| **WobbleCard** | Card that wobbles/shakes on interaction. | MEDIUM |
| **ExpandableCard** | Card that expands to reveal more content. | HIGH |
| **CardSpotlight** | Card with spotlight effect following cursor. | MEDIUM |
| **FocusCards** | Cards that focus/blur based on cursor position. | MEDIUM |
| **InfiniteMovingCards** | Horizontally scrolling infinite card carousel. | MEDIUM |
| **DraggableCard** | Card that can be dragged and dropped. | HIGH |
| **CometCard** | Card with comet trail effect on hover. | LOW |
| **GlareCard** | Card with glare/shine effect on hover. | MEDIUM |
| **DirectionAwareHover** | Card with direction-aware hover animation. | MEDIUM |
| **MagicCard** | Card with magical particle effects. | LOW |
| **NeonGradientCard** | Card with neon gradient border glow. | MEDIUM |
| **FlippingCard** | Card that flips to reveal back content. | MEDIUM |
| **AnimatedCard** | Card with entrance/exit animations. | MEDIUM |

#### Backgrounds & Effects
| Component | Intent | Priority |
|---|---|---|
| **AnimatedBeam** | Animated light beam across background. | MEDIUM |
| **BorderBeam** | Animated beam along border. | MEDIUM |
| **ShineBorder** | Border with animated shine effect. | MEDIUM |
| **Meteors** | Animated meteor shower effect. | LOW |
| **Confetti** | Confetti particle explosion effect. | MEDIUM |
| **Particles** | Interactive particle system background. | MEDIUM |
| **Sparkles** | Sparkle particle effect. | LOW |
| **Spotlight** | Spotlight effect following cursor. | MEDIUM |
| **TracingBeam** | Tracing beam that follows scroll. | LOW |
| **LampEffect** | Lamp/light source effect. | LOW |
| **Vortex** | Vortex/swirl background effect. | LOW |
| **AuroraBackground** | Aurora borealis animated background. | MEDIUM |
| **WavyBackground** | Wavy animated background. | LOW |
| **BackgroundBoxes** | Animated background boxes pattern. | LOW |
| **BackgroundBeams** | Animated beams in background. | MEDIUM |
| **BackgroundLines** | Animated lines in background. | LOW |
| **BackgroundRippleEffect** | Ripple effect on background. | LOW |
| **DottedGlowBackground** | Dotted pattern with glow effect. | LOW |
| **GradientAnimation** | Animated gradient background. | MEDIUM |
| **FlickeringGrid** | Flickering grid background. | LOW |
| **AnimatedGridPattern** | Animated grid pattern background. | LOW |
| **RetroGrid** | Retro-style grid background. | LOW |
| **Ripple** | Ripple effect background. | LOW |
| **DotPattern** | Dot pattern background. | LOW |
| **GridPattern** | Grid pattern background. | LOW |
| **HexagonPattern** | Hexagon pattern background. | LOW |
| **StripedPattern** | Striped pattern background. | LOW |
| **InteractiveGridPattern** | Interactive grid that responds to cursor. | LOW |
| **LightRays** | Light rays effect. | LOW |
| **NoiseTexture** | Noise texture background. | LOW |
| **DitherShader** | Dither shader effect. | LOW |
| **WebcamPixelGrid** | Webcam feed as pixel grid. | LOW |
| **ParallaxHeroImages** | Parallax scrolling hero images. | MEDIUM |
| **Scales** | Scale/zoom effect on scroll. | LOW |
| **GlowingEffect** | Glowing effect on hover. | MEDIUM |
| **GoogleGeminiEffect** | Gemini-style glowing effect. | LOW |
| **CanvasRevealEffect** | Canvas-based reveal animation. | LOW |
| **SvgMaskEffect** | SVG mask reveal effect. | LOW |
| **WarpBackground** | Warp/distortion background. | LOW |
| **HyperspaceBackground** | Hyperspace warp background. | LOW |
| **StripeAnimatedGradient** | Animated gradient stripes. | LOW |
| **PixelDistortion** | Pixel distortion effect. | LOW |
| **PulseShader** | Pulsing shader effect. | LOW |
| **MouseWave** | Wave effect following mouse. | LOW |

#### Buttons
| Component | Intent | Priority |
|---|---|---|
| **MagneticButton** | Button that magnetically attracts to cursor. | MEDIUM |
| **ShimmerButton** | Button with shimmer/sweep animation. | HIGH |
| **RippleButton** | Button with material ripple effect. | HIGH |
| **RainbowButton** | Button with rainbow gradient border. | LOW |
| **MovingBorder** | Button with animated moving border. | MEDIUM |
| **HoverBorderGradient** | Button with gradient border on hover. | MEDIUM |
| **StatefulButton** | Button with loading/success/error states. | HIGH |
| **NoiseBackground** | Button with noise texture background. | LOW |
| **GlowingButton** | Button with glow effect. | MEDIUM |
| **SwipeButton** | Button with swipe-to-confirm interaction. | MEDIUM |
| **GradientSlideButton** | Button with gradient slide animation. | LOW |
| **StarButton** | Button with star rating interaction. | LOW |
| **ConfettiButton** | Button that triggers confetti on click. | MEDIUM |
| **ShuffleButton** | Button with shuffle animation. | LOW |
| **StaggerButton** | Button with staggered letter animation. | LOW |
| **LikeButton** | Button with like/heart animation. | MEDIUM |
| **PulsatingButton** | Button with pulsing animation. | LOW |
| **InteractiveHoverButton** | Button with interactive hover effect. | LOW |
| **AnimatedThemeToggler** | Theme toggle with animation. | MEDIUM |
| **ShinyButton** | Button with shiny reflection effect. | LOW |
| **Backlight** | Button with backlight glow effect. | LOW |

#### Loaders
| Component | Intent | Priority |
|---|---|---|
| **MultiStepLoader** | Multi-step progress loader with labels. | HIGH |
| **Loader** | Animated loading spinner with variants. | HIGH |

#### Navigation
| Component | Intent | Intent | Priority |
|---|---|---|---|
| **FloatingNavbar** | Floating navigation bar with blur backdrop. | HIGH |
| **NavbarMenu** | Animated navbar with dropdown menus. | HIGH |
| **FloatingDock** | macOS-style floating dock. | HIGH |
| **Notch** | Mobile-style notch navigation. | MEDIUM |
| **ResizableNavbar** | Resizable navigation bar. | LOW |
| **StickyBanner** | Sticky announcement banner. | MEDIUM |

#### Inputs & Forms
| Component | Intent | Priority |
|---|---|---|
| **PlaceholdersAndVanishInput** | Input with animated placeholder that vanishes. | MEDIUM |
| **GooeyInput** | Input with gooey/blob animation effect. | LOW |
| **FileUpload** | Drag-and-drop file upload with progress. | HIGH |
| **SignupForm** | Complete signup form with validation. | MEDIUM |

#### Overlays & Popovers
| Component | Intent | Priority |
|---|---|---|
| **AnimatedModal** | Modal with entrance/exit animations. | HIGH |
| **AnimatedTooltip** | Tooltip with smooth animations. | MEDIUM |
| **LinkPreview** | Hover preview for links. | MEDIUM |

#### Carousels & Sliders
| Component | Intent | Priority |
|---|---|---|
| **ImagesSlider** | Image slider with transitions. | HIGH |
| **Carousel** | Generic carousel with navigation. | HIGH |
| **AppleCardsCarousel** | Apple-style card carousel. | MEDIUM |
| **AnimatedTestimonials** | Auto-scrolling testimonial carousel. | MEDIUM |
| **Marquee** | Horizontal scrolling marquee. | HIGH |
| **ImagesBadge** | Image with floating badge. | LOW |

#### Layout & Grid
| Component | Intent | Priority |
|---|---|---|
| **BentoGrid** | Bento-style grid layout. | HIGH |
| **LayoutGrid** | Grid layout with animated cells. | MEDIUM |
| **ContainerCover** | Container with cover image effect. | LOW |
| **3DWrapper** | 3D perspective wrapper for any content. | MEDIUM |
| **3DPin** | 3D pinned element. | LOW |
| **3DMarquee** | 3D marquee effect. | LOW |
| **3DGlobe** | 3D interactive globe. | LOW |
| **3DBook** | 3D book with page flip. | LOW |
| **AnimatedCollection** | Animated collection/grid of items. | MEDIUM |
| **FluidExpandingGrid** | Grid that fluidly expands on interaction. | LOW |
| **MagnifiedBento** | Bento grid with magnification on hover. | LOW |

#### Data & Visualization
| Component | Intent | Priority |
|---|---|---|
| **GitHubGlobe** | 3D globe showing GitHub activity. | LOW |
| **WorldMap** | Interactive world map. | LOW |
| **Timeline** | Vertical timeline with animated entries. | HIGH |
| **Compare** | Side-by-side comparison slider. | MEDIUM |
| **Codeblock** | Syntax-highlighted code block with copy. | HIGH |
| **CodeComparison** | Side-by-side code diff comparison. | MEDIUM |
| **DottedMap** | Dotted world map visualization. | LOW |
| **ContributionGraph** | GitHub-style contribution heatmap. | MEDIUM |
| **Gantt** | Gantt chart component. | MEDIUM |
| **Kanban** | Kanban board with drag-and-drop. | HIGH |
| **ScrollProgress** | Scroll progress indicator. | MEDIUM |
| **AnimatedCircularProgressBar** | Animated circular progress. | MEDIUM |

#### Cursor & Pointer
| Component | Intent | Priority |
|---|---|---|
| **FollowingPointer** | Custom cursor that follows mouse. | LOW |
| **PointerHighlight** | Highlight effect around cursor. | LOW |
| **Lens** | Magnifying lens effect on hover. | MEDIUM |
| **SmoothCursor** | Smooth/custom cursor animation. | LOW |
| **CursorCards** | Cards that respond to cursor position. | MEDIUM |

#### Device Mocks
| Component | Intent | Priority |
|---|---|---|
| **Safari** | Safari browser mockup. | MEDIUM |
| **iPhone** | iPhone device mockup. | MEDIUM |
| **Android** | Android device mockup. | MEDIUM |
| **Terminal** | Terminal window mockup. | MEDIUM |
| **Keyboard** | Animated keyboard display. | LOW |

#### Scroll & Parallax
| Component | Intent | Priority |
|---|---|---|
| **ParallaxScroll** | Parallax scrolling effect. | MEDIUM |
| **StickyScrollReveal** | Sticky scroll with reveal animation. | MEDIUM |
| **MacbookScroll** | Scroll-triggered MacBook reveal. | LOW |
| **ContainerScrollAnimation** | Container-based scroll animation. | LOW |
| **HeroParallax** | Hero section with parallax. | MEDIUM |

#### Special / Misc
| Component | Intent | Priority |
|---|---|---|
| **OrbitingCircles** | Orbiting circles animation. | LOW |
| **AvatarCircles** | Stacked avatar circles. | MEDIUM |
| **IconCloud** | Cloud of icons. | LOW |
| **TweetCard** | Tweet-style card. | LOW |
| **FileTree** | File tree explorer. | MEDIUM |
| **Sandbox** | Code sandbox/preview. | MEDIUM |
| **Snippet** | Code snippet display. | MEDIUM |
| **Choicebox** | Choice/selection box. | LOW |
| **InlineEdit** | Inline editable text. | MEDIUM |
| **FilterInteraction** | Animated filter UI. | LOW |
| **DynamicToolbar** | Context-aware dynamic toolbar. | LOW |
| **EmptyTestimonial** | Empty state for testimonials. | LOW |
| **FeatureCarousel** | Feature highlight carousel. | MEDIUM |
| **FolderInteraction** | Interactive folder expand/collapse. | LOW |
| **DeleteButton** | Button with delete confirmation animation. | MEDIUM |
| **DiscoverButton** | Button with discover/reveal animation. | LOW |
| **DiscreteTabs** | Discrete/segmented tabs. | MEDIUM |
| **Day Picker** | Day-of-week picker. | MEDIUM |
| **Bucket** | Bucket/container component. | LOW |
| **BottomMenu** | Bottom sheet menu. | MEDIUM |
| **ExpandableGallery** | Expandable image gallery. | MEDIUM |
| **ImageSplit** | Split image comparison. | LOW |
| **ImageTrail** | Image trail effect on hover. | LOW |
| **InfiniteRibbon** | Infinite scrolling ribbon. | LOW |
| **SocialProofAvatars** | Social proof avatar stack. | MEDIUM |
| **AvatarStack** | Stacked avatars with hover animation. | MEDIUM |
| **CoolMode** -- Fun/cool mode toggle. | LOW |
| **PixelImage** | Pixelated image effect. | LOW |
| **NeonGradientCard** | Neon gradient card effect. | LOW |
| **Backlight** | Backlight glow effect. | LOW |
| **ScrollProgress** | Scroll progress bar. | MEDIUM |
| **AnimatedThemeToggler** | Animated theme toggle. | MEDIUM |
| **Confetti** | Confetti explosion effect. | MEDIUM |
| **Particles** | Particle system. | MEDIUM |
| **Meteors** | Meteor shower effect. | LOW |
| **Sparkles** | Sparkle particles. | LOW |
| **GlareHover** | Glare effect on hover. | MEDIUM |
| **ShineBorder** | Shine border effect. | MEDIUM |
| **BorderBeam** | Border beam animation. | MEDIUM |
| **AnimatedBeam** | Animated beam effect. | MEDIUM |
| **MagicCard** | Magic card effect. | LOW |
| **WarpBackground** | Warp background effect. | LOW |
| **HyperspaceBackground** | Hyperspace background. | LOW |
| **StripeAnimatedGradient** | Animated gradient stripes. | LOW |
| **PixelDistortion** | Pixel distortion effect. | LOW |
| **PulseShader** | Pulse shader effect. | LOW |
| **MouseWave** | Mouse wave effect. | LOW |
| **BlurReveal** | Blur reveal text effect. | MEDIUM |
| **FadeUpWord** | Word fade-up animation. | MEDIUM |
| **StaggerBlurEffect** | Staggered blur effect. | LOW |

---

## 3. IMPROVEMENTS TO EXISTING CVKG COMPONENTS

### HuginChat (agent chat)
- Add streaming message support with `TokenStream` integration
- Add tool call display with `ToolCard` component
- Add markdown rendering with code highlighting
- Add copy-to-clipboard on messages
- Add message actions (retry, edit, delete)
- Add typing indicator
- Add file attachment support

### TokenStream (streaming text)
- Add word-level animation (not just char-by-char)
- Add typing indicator cursor
- Add speed control

### GeriPrompt (prompt input)
- Add slash commands dropdown
- Add file attachment button
- Add model selector dropdown
- Add voice input button

### BifrostTabs (glass tabs)
- Add animated sliding indicator
- Add close button on tabs
- Add drag-to-reorder
- Add scroll arrows for overflow

### HeimdallDock (dock)
- Add magnification animation on hover
- Add bounce animation on click
- Add badge notifications
- Add context menu on right-click
- Add drag-to-reorder

### NiflheimSidebar (sidebar)
- Add collapsible sections
- Add search/filter
- Add drag-to-resize
- Add nested navigation
- Add active indicator animation

### ValkyrieToolbar (toolbar)
- Add overflow menu for small screens
- Add tooltips on hover
- Add keyboard shortcut hints
- Add context-aware visibility

### Toast (notifications)
- Add Sonner-style stacking
- Add swipe-to-dismiss
- Add action buttons
- Add progress bar
- Add position variants (6 positions)

### Popover
- Add arrow/pointer
- Add animation variants
- Add focus trap
- Add escape-to-close

### Dialog/Modal
- Add animation variants (fade, scale, slide)
- Add drag-to-move
- Add resize handles
- Add fullscreen toggle

### Select/Dropdown
- Add multi-select
- Add search/filter within options
- Add virtual scrolling for large lists
- Add option groups
- Add clear button

### Input
- Add prefix/suffix icons
- Add character counter
- Add validation states with icons
- Add password visibility toggle
- Add clear button

### Table
- Add column resizing
- Add column reordering
- Add row selection
- Add sorting indicators
- Add pagination
- Add sticky headers
- Add expandable rows

### Calendar
- Add range selection
- Add multi-date selection
- Add disabled dates
- Add min/max date constraints
- Add month/year navigation

### DatePicker
- Add time picker
- Add range selection
- Add preset ranges (today, yesterday, last 7 days)
- Add clear button

### Slider
- Add range slider (two thumbs)
- Add step markers
- Add tooltip with current value
- Add vertical orientation

### Progress
- Add circular progress
- Add animated stripes
- Add percentage label
- Add indeterminate state

### Tabs
- Add animated indicator
- Add close buttons
- Add scroll arrows for overflow
- Add vertical tabs
- Add drag-to-reorder

### Accordion
- Add animation variants
- Add multiple/single expand mode
- Add icon rotation animation
- Add nested accordions

### Tooltip
- Add delay variants
- Add arrow/pointer
- Add rich content (not just text)
- Add follow-cursor mode

### Avatar
- Add status indicator
- Add fallback initials
- Add image loading state
- Add group/stack variant

### Badge
- Add dot variant
- Add pulse animation
- Add dismiss button
- Add count variant

### Card
- Add hover lift effect
- Add loading skeleton state
- Add selected state
- Add expandable content

### Alert
- Add icon variants
- Add dismiss button
- Add action buttons
- Add animation entrance/exit

### Skeleton
- Add shimmer animation
- Add pulse animation
- Add custom shapes (circle, rect, text)
- Add wave animation

### Spinner
- Add size variants
- Add color variants
- Add speed variants
- Add label support

---

## 4. IMPLEMENTATION PRIORITY

### Phase 1: High-Impact Agent Components (Week 1)
1. AgentChat (full chat surface)
2. MessageList (scrollable messages)
3. InputBar (text input + send/stop)
4. UserMessage / AssistantMessage (message bubbles)
5. Markdown (rendered markdown)
6. ToolCard (tool call display)
7. SuggestionChips (quick actions)
8. ModelPicker (model selector)
9. CopyToolbar (copy/share actions)
10. TextShimmer (loading placeholder)

### Phase 2: High-Impact UI Primitives (Week 2)
1. BentoGrid (bento layout)
2. AnimatedList (animated list items)
3. ExpandableCard (expandable card)
4. FileUpload (drag-and-drop upload)
5. CodeBlock (syntax-highlighted code)
6. Timeline (vertical timeline)
7. NumberTicker (animated counter)
8. AvatarStack (stacked avatars)
9. Kanban (drag-and-drop board)
10. Carousel (image/content carousel)

### Phase 3: Animation & Effects (Week 3)
1. TextAnimate (text animations)
2. TypewriterEffect (typewriter)
3. CardHoverEffect (3D tilt)
4. ShimmerButton (shimmer button)
5. RippleButton (ripple effect)
6. ConfettiButton (confetti effect)
7. Spotlight (cursor spotlight)
8. ParallaxScroll (parallax)
9. AnimatedModal (animated modal)
10. Marquee (scrolling marquee)

### Phase 4: Polish & Remaining (Week 4)
1. Improve existing components (see Section 3)
2. Add remaining text animations
3. Add remaining card effects
4. Add remaining backgrounds
5. Add device mocks (Safari, iPhone, Terminal)
6. Add cursor effects (Lens, Pointer)
7. Add remaining loaders
8. Add remaining navigation patterns

---

## 5. DESIGN PRINCIPLES FOR NEW COMPONENTS

1. **Composable**: Every component should be built from smaller primitives. AgentChat composes MessageList + InputBar + Suggestions.

2. **Themeable**: All colors, sizes, and spacing must use the design token system (FONT_*, SPACE_*, RADIUS_*, Color). No hardcoded values.

3. **Accessible**: WCAG 2.1 AA compliance. Focus states, ARIA labels, keyboard navigation, screen reader support.

4. **Performant**: Animations use GPU-accelerated properties (transform, opacity). No layout thrashing. Virtual scrolling for large lists.

5. **Type-safe**: Full Rust type safety. Builder patterns for optional props. No unwrap() in production code.

6. **Consistent API**: All components follow the same pattern:
   - `new()` constructor
   - Builder methods: `fn prop(mut self, value: T) -> Self`
   - `View` trait implementation
   - `render(&self, renderer: &mut dyn Renderer, rect: Rect)`

7. **Zero dependencies**: All components use only cvkg-core Renderer API. No external crates.

8. **Testable**: Each component should have at least one test verifying it renders without panicking.

---

## 6. CULT UI COMPONENTS (from cult-ui.com)

### Marketing & Heroes
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **HeroDithering** | Hero section with dithering shader effect | No | MEDIUM |
| **HeroColorPanels** | Hero with animated color panel grid | No | HIGH |
| **HeroHeatmap** | Hero with heatmap visualization | No | MEDIUM |
| **HeroLiquidMetal** | Hero with liquid metal shader | No | MEDIUM |
| **HeroStaticRadialGradient** | Hero with static radial gradient | No | LOW |
| **BgMediaHero** | Hero with video/image background | No | HIGH |
| **LogoCarousel** | Animated logo carousel (marquee) | No | HIGH |
| **TweetGrid** | Grid of embedded tweets | No | LOW |
| **GradientHeading** | Heading with animated gradient text | No | MEDIUM |

### Buttons
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **NeumorphButton** | Neumorphic/soft UI button | No | MEDIUM |
| **TextureButton** | Button with texture background | No | LOW |
| **BgAnimateButton** | Button with animated background | No | MEDIUM |
| **BorderBeamButton** | Button with animated border beam | Partial (have BorderBeam) | MEDIUM |
| **MetalButton** | Metallic finish button | No | LOW |
| **CosmicButton** | Button with cosmic/space effect | No | LOW |
| **GradientButtonGroup** | Group of gradient buttons | No | MEDIUM |

### Expandable Widgets
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **DynamicIsland** | iOS Dynamic Island-style expandable | No | HIGH |
| **Onboarding** | Step-by-step onboarding flow | No | HIGH |
| **FamilyButton** | Button that expands to show family/related actions | No | MEDIUM |
| **ToolbarExpandable** | Toolbar that expands on interaction | No | MEDIUM |
| **ExpandableScreen** | Screen that expands from a trigger | No | MEDIUM |
| **ExpandableCard** | Card that expands to show more | Partial | HIGH |
| **MorphSurface** | Surface that morphs shape on interaction | No | LOW |
| **SidePanel** | Side panel that slides in | No | HIGH |
| **FamilyDrawer** | Drawer with family/related items | No | MEDIUM |
| **IntroDisclosure** | Progressive disclosure for introductions | No | MEDIUM |

### Cards & Surfaces
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **MinimalCard** | Minimal card design | Partial (have Card) | MEDIUM |
| **CutoutCard** | Card with cutout design | No | LOW |
| **NeumorphEyebrow** | Neumorphic eyebrow/label | No | LOW |
| **TextureCard** | Card with texture background | No | LOW |
| **ShiftCard** | Card that shifts on interaction | No | LOW |

### Frames & Mockups
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **BrowserWindow** | Browser window mockup frame | No | HIGH |
| **CodeBlock** | Enhanced code block with syntax highlighting | Partial | HIGH |
| **TerminalAnimation** | Animated terminal window | No | MEDIUM |

### Textures & Overlays
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **TextureOverlay** | Texture overlay effect | No | LOW |
| **DistortedGlass** | Distorted glass effect | No | MEDIUM |
| **BackgroundTexture** | Textured background | No | LOW |
| **EdgeBlur** | Blur effect at edges | No | MEDIUM |
| **DitherImage** | Dithering effect on images | No | LOW |

### Visual Systems
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **GridBeam** | Grid with beam animation | No | MEDIUM |
| **FractalGrid** | Fractal grid pattern | No | LOW |
| **CanvasFractalGrid** | Canvas-rendered fractal grid | No | LOW |
| **StripeBgGuides** | Striped background guides | No | LOW |
| **LightBoard** | Light board/grid effect | No | LOW |
| **ShaderLensBlur** | Shader-based lens blur | No | LOW |
| **SVGShapes** | SVG shape decorations | No | MEDIUM |
| **SVGShapesAnimated** | Animated SVG shapes | No | MEDIUM |
| **SVGBands** | SVG band decorations | No | LOW |

### Navigation & Floating UI
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **DirectionAwareTabs** | Tabs that adapt to direction | No | MEDIUM |
| **FloatingPanel** | Floating panel that follows cursor | No | MEDIUM |
| **Popover** | Enhanced popover | Partial | MEDIUM |
| **PopoverForm** | Popover with form content | No | MEDIUM |
| **MacOSDock** | macOS-style dock | Partial (have HeimdallDock) | MEDIUM |

---

## 7. MUI X COMPONENTS (from mui.com/x)

### Data Grid Enhancements (we have basic Table/RunesTable)
| Feature | Intent | Have? | Priority |
|---|---|---|---|
| **Charts Integration** | Embed charts in data grid cells | No | HIGH |
| **AI Assistant** | AI-powered data analysis in grid | No | HIGH |
| **Undo/Redo** | Undo/redo for grid edits | Partial | HIGH |
| **Drag Fill** | Drag to fill cells (like Excel) | No | MEDIUM |
| **LongText Column** | Column type for long text with expand | No | MEDIUM |
| **Server-side Pivoting** | Pivot tables with server data | No | MEDIUM |
| **Row Grouping** | Group rows with adaptive exploration | No | MEDIUM |
| **Export Resilience** | Robust export (CSV, Excel, PDF) | No | MEDIUM |
| **Smoother Reordering** | Drag-to-reorder with clear affordances | No | MEDIUM |
| **Pinned Areas** | Pinned columns/rows with scroll | No | HIGH |
| **Data Source** | Server-side data source with editing | No | HIGH |
| **Toolbar** | Built-in toolbar with actions | No | MEDIUM |
| **No Columns Overlay** | Empty state when no columns | No | LOW |

### Charts (we have basic charts via gpu_charts)
| Feature | Intent | Have? | Priority |
|---|---|---|---|
| **Candlestick** | Financial candlestick charts | No | HIGH |
| **Range Bar** | Range bar charts | No | MEDIUM |
| **Sankey** | Sankey flow diagrams | No | MEDIUM |
| **Funnel** | Funnel charts | No | MEDIUM |
| **Radar** | Radar/spider charts | No | MEDIUM |
| **Heatmap** | WebGL heatmap renderer | No | MEDIUM |
| **Zoom/Pan** | Interactive zoom and pan | No | HIGH |
| **Brush Selection** | Brush to select data range | No | MEDIUM |
| **Keyboard Nav** | Keyboard navigation for charts | No | MEDIUM |
| **Animation Engine** | Smooth chart animations | No | MEDIUM |
| **SSR** | Server-side rendering support | No | LOW |

### Date/Time Pickers (we have basic DatePicker)
| Feature | Intent | Have? | Priority |
|---|---|---|---|
| **Time Range Picker** | Pick a time range | No | HIGH |
| **Better Range Defaults** | Smart default ranges | No | MEDIUM |
| **Polished Inputs** | Cross-device input polish | No | MEDIUM |
| **Accessible DOM** | Accessible DOM structure | No | MEDIUM |
| **Keyboard Editing** | Keyboard editing on mobile | No | MEDIUM |

### Tree View (we have basic VTree)
| Feature | Intent | Have? | Priority |
|---|---|---|---|
| **Virtualization** | Virtual scrolling for large trees | No | HIGH |
| **Drag-and-Drop** | Drag to reorder tree nodes | No | HIGH |
| **Lazy Loading** | Load children on demand | No | HIGH |
| **Selection Propagation** | Auto-select parents/children | No | MEDIUM |
| **Customization Hook** | Hook for custom tree behavior | No | MEDIUM |

### Scheduler (NEW)
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **Scheduler** | Full calendar scheduler with events | No | HIGH |
| **Timeline View** | Timeline view for scheduler | No | HIGH |
| **Resource View** | Resource/room booking view | No | MEDIUM |

### Chat (NEW)
| Component | Intent | Have? | Priority |
|---|---|---|---|
| **Chat** | Full chat component (MUI X) | Partial (have HuginChat) | HIGH |

---

## 8. JOY UI COMPONENTS (from v7.mui.com/joy-ui)

### Key Design Principles (Joy Design)
- **Beautiful out of the box**: Thoughtfully crafted defaults
- **Highly customizable**: CSS variables for every piece
- **Developer experience**: Sparks joy in building
- **Global variants**: Consistent variant system across all components
- **Color inversion**: Automatic color inversion for dark mode
- **Automatic adjustment**: Components adjust to context
- **Dark mode optimization**: First-class dark mode support

### Joy UI Component Categories
| Category | Components | Have? |
|---|---|---|
| **Inputs** | Input, Textarea, Select, Checkbox, Radio, Switch, Slider, Autocomplete | Partial |
| **Data Display** | Table, List, Badge, Chip, Avatar, Accordion, Tooltip | Partial |
| **Feedback** | Alert, Progress, Modal, Drawer, Snackbar | Partial |
| **Navigation** | Menu, Tabs, Breadcrumb, Link, Pagination, Stepper | Partial |
| **Layout** | Box, Stack, Grid, Container, Divider | Partial |
| **Surfaces** | Card, Sheet, Accordion | Partial |
| **Buttons** | Button, Icon Button, Button Group | Partial |
| **Typography** | Typography, Link | Partial |

### Joy UI Unique Features (not in standard shadcn)
| Feature | Intent | Have? | Priority |
|---|---|---|---|
| **Global Variants** | Consistent variant system (solid, soft, outlined, plain) | No | HIGH |
| **Color Inversion** | Automatic color inversion for dark mode | No | HIGH |
| **Automatic Adjustment** | Components adjust to context automatically | No | MEDIUM |
| **Joy Design Tokens** | Design token system for Joy Design | Partial | HIGH |
| **CSS Variables** | Every component customizable via CSS vars | No | MEDIUM |
| **Theme Overrides** | Per-component theme overrides | No | MEDIUM |
| **Variant Prop** | `variant` prop on every component | No | HIGH |
| **Size Prop** | `size` prop on every component (xs, sm, md, lg) | No | HIGH |
| **Color Prop** | `color` prop on every component (primary, neutral, danger, info, success, warning) | No | HIGH |
| **Start/End Decoration** | `startDecorator` and `endDecorator` props | No | MEDIUM |
| **Component Slots** | Slot-based customization system | No | MEDIUM |

---

## 9. UPDATED IMPLEMENTATION PRIORITY

### Phase 1: High-Impact Agent + Data (Week 1)
1. AgentChat, MessageList, InputBar, UserMessage, AssistantMessage
2. Markdown renderer, ToolCard, SuggestionChips, ModelPicker
3. BrowserWindow mockup, CodeBlock (enhanced), TerminalAnimation
4. HeroColorPanels, BgMediaHero, LogoCarousel

### Phase 2: Enhanced Primitives + Joy UI Patterns (Week 2)
1. Global variant system (solid, soft, outlined, plain)
2. Color inversion for dark mode
3. Size/Color props on all components
4. Start/End decoration support
5. DynamicIsland, Onboarding flow
6. SidePanel, ExpandableCard (enhanced)

### Phase 3: Data Grid + Charts (Week 3)
1. Data Grid: pinned columns, row grouping, undo/redo, toolbar
2. Charts: candlestick, range bar, funnel, radar, heatmap
3. Tree View: virtualization, drag-and-drop, lazy loading
4. Scheduler component
5. Date/Time: time range picker, better defaults

### Phase 4: Animation + Effects (Week 4)
1. TextAnimate, TypewriterEffect, NumberTicker
2. CardHoverEffect, ShimmerButton, RippleButton
3. ConfettiButton, Spotlight, ParallaxScroll
4. AnimatedModal, Marquee
5. Texture effects, SVG shapes, GridBeam

### Phase 5: Remaining Polish (Week 5)
1. Remaining text animations
2. Remaining card effects
3. Remaining backgrounds
4. Device mocks (Safari, iPhone, Android)
5. Cursor effects (Lens, Pointer)
6. Remaining loaders and navigation

---

## 10. MATERIAL 3 COMPONENTS (from m3.material.io)

### Component Parity Summary

**33 M3 components reviewed. CVKG has 22 (67%), partial 5 (15%), missing 6 (18%).**

#### Missing M3 Components (6):
| Component | Intent | Priority |
|---|---|---|
| **FAB (Floating Action Button)** | Circular, fixed-position primary action button | HIGH |
| **Extended FAB** | FAB with label text for clarity | HIGH |
| **FAB Menu** | FAB that expands to show related actions | MEDIUM |
| **Time Picker** | Clock face or dial time input | HIGH |
| **Date Range Picker** | Calendar for selecting date ranges | HIGH |
| **Filter/Input/Assist Chips** | 4 chip types for filtering, input, suggestions | MEDIUM |

#### Partial M3 Components (need improvement):
| Component | What's Missing | Priority |
|---|---|---|
| **Button** | Tonal button variant, proper elevated variant | HIGH |
| **Radio Button** | Standalone radio (only exists inside Checkbox) | MEDIUM |
| **Chips** | Assist, filter, input, suggestion variants | MEDIUM |
| **Navigation** | Navigation rail, bottom nav bar | MEDIUM |
| **Progress** | Linear progress indicator | MEDIUM |

### Material 3 Design System Features (not in CVKG):
| Feature | Description | Priority |
|---|---|---|
| **Dynamic Color** | Color extraction from wallpaper/image | HIGH |
| **Elevation System** | 0-5 levels with shadow + surface tint | HIGH |
| **State Layers** | Hover/focus/pressed/disabled overlays | HIGH |
| **Surface Roles** | Surface, surface-variant, surface-container hierarchy | MEDIUM |
| **Color Roles** | Primary/secondary/tertiary/error/neutral/neutral-variant | MEDIUM |
| **Typography Scale** | Display/headline/title/body/label (L/M/S) | HIGH |
| **Motion System** | Emphasized/standard easing curves | MEDIUM |
| **Shape System** | 7-level rounded corner system | LOW |
