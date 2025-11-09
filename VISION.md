# Felt UI: A Vision for Creative Tools

## The Problem

Most UI frameworks make a fundamental tradeoff: they abstract away the GPU to simplify common use cases, but in doing so, they eliminate the very capabilities that creative tools need.

I discovered this when using **gpui** (by Zed) for my project. Despite being modern and well-designed, it doesn't expose custom shaders or low-level GPU access. For creative tools, this is a dealbreaker:

- Can't write custom WGSL shaders for effects
- Can't apply real-time image processing
- Can't mix 2D UI with custom rendering passes
- No access to the GPU pipeline for advanced techniques

This abstraction works for text editors and business apps, but creative tools need both **great UI** and **GPU freedom**.

## The Solution: Felt UI

Felt UI will give users **both** efficient 2D UI rendering **and** complete GPU freedom through a two-layer architecture.

### Architecture

**Layer 1: Vello for UI primitives**

Vello will handle standard 2D rendering efficiently:
- Rectangles, circles, paths, and vector graphics
- Text rendering via Parley (shaping, layout, bidirectional text)
- Gradients, fills, and strokes
- GPU-accelerated with excellent performance

**Layer 2: Direct wgpu access for custom work**

The GPU pipeline stays accessible for advanced needs:
- Write custom WGSL shaders
- Create custom render pipelines
- Mix 3D rendering with 2D UI
- Apply post-processing effects (bloom, blur, color grading)
- Image filters and real-time effects
- Compute shaders for heavy processing

**The key insight:** These layers compose. Users can render UI with Vello, then apply custom shaders on top, then render more UI. The entire GPU pipeline is available, not hidden behind abstractions.

### Why wgpu?

I chose **wgpu** over platform-specific APIs (Metal, DirectX, Vulkan) because:

**Cross-platform by default:**
- macOS → Metal
- Windows → DirectX 12 / Vulkan
- Linux → Vulkan

**Better Developer Experience:**
- While native APIs (Metal, DirectX 12, Vulkan) eliminate abstraction overhead, wgpu's performance is excellent and the DX benefits far outweigh the <5% theoretical cost
- Users would have to handle all platforms separately with native APIs
- One shader language (WGSL) works everywhere
- One API to learn
- Minimal overhead, maximum portability

**Industry momentum:**
- Used by Bevy, egui, and other Rust projects
- Active development
- Good tooling and documentation

## What This Enables

**Composable Rendering Pipelines**

Imagine rendering your UI to a texture with Vello, then applying a custom bloom shader for a glowing effect, then rendering more UI on top. Each pass uses the right tool - Vello for UI, custom shaders for effects - and they compose naturally.

**Mixing 2D and 3D**

A 3D modeling tool could render the 3D viewport with custom shaders, overlay Vello-based UI panels and toolbars, then apply post-processing like anti-aliasing or color grading. The entire pipeline is under user control.

**Real-time Effects**

An image editor could display an image with live shader-based filters (brightness, contrast, saturation), render pixel-perfect UI controls with Vello, and update everything at 120fps. No compromise between UI quality and effect performance.

**Custom Visualizations**

A data visualization tool could use compute shaders for heavy processing, render results with custom pipelines, then overlay interactive UI elements with Vello. GPU acceleration where needed, great UI everywhere else.

## What Makes This Different

Most UI frameworks optimize for one use case: business applications. They abstract away complexity, which means hiding capability.

**Felt UI takes a different approach:**

Instead of hiding the GPU, we expose it. Instead of making decisions for users, we give them building blocks. Instead of "one size fits all," we enable creative tools to be creative.

This means:
- **No artificial limitations** - If the GPU can do it, users can do it
- **Composability over convenience** - Powerful primitives that work together
- **Performance by design** - Both Vello and wgpu are GPU-native
- **Cross-platform by default** - Write once, run everywhere

## Who This Is For

Felt UI is being built for developers creating:

- **Image and video editors** - Real-time effects, filters, and processing
- **3D tools** - Modeling, rendering, CAD applications
- **Audio tools** - Waveform visualization, spectral analysis
- **Game development tools** - Level editors, asset browsers, particle editors
- **Data visualization** - Scientific computing, real-time analytics
- **Creative coding environments** - Generative art, live coding, VJ tools

If your application needs both **great UI** and **GPU power**, Felt UI is for you.

## Design Philosophy

> **"Give users powerful building blocks, don't make decisions for them."**

Felt UI is built on three principles:

**1. Capability over Convenience**
We won't hide features to make the simple case easier. Instead, we'll provide powerful primitives that compose well. Convenience can be built on top of capability, but not vice versa.

**2. Performance by Default**
Both layers - Vello for 2D and wgpu for custom work - are GPU-native. There's no CPU fallback, no "compatibility mode." If users target modern GPUs, Felt UI will use them fully.

**3. Trust Developers**
We assume users know what they're doing. Want to write a custom shader? Here's the GPU. Want efficient 2D rendering? Here's Vello. Want to mix them? Go ahead. We provide tools, not guardrails.

## Why I'm Building This

I started this project because I needed a UI framework that didn't compromise. One that could handle standard UI efficiently while still letting me access the GPU for custom rendering.

Existing frameworks made me choose: either great UI with no GPU access, or full GPU control with no UI primitives. Felt UI will give developers both.

This isn't just about features - it's about recognizing that creative tools have different needs than business applications. They need to be beautiful, fast, and unrestricted. That's what Felt UI will deliver.

---

## Implementation Exploration: Layering Approaches

A key architectural decision is how to enable mixing Vello UI rendering with custom GPU work. There are two main approaches to consider:

### Approach 1: Global Layer System

Some frameworks (like sol-ui) use numbered global layers:

```rust
layers.add_raw_layer(0, options, |ctx| { /* starfield shader */ });
layers.add_ui_layer(1, options, || { /* UI widgets */ });
```

**How it works:** Layers are numbered globally (0, 1, 2...) and rendered in order. Raw GPU work goes in "raw layers," standard UI in "UI layers."

**Advantages:**
- Simple mental model
- Clear separation between rendering modes
- Good for full-screen backgrounds or post-processing

**Limitations:**
- Global coordination required - every part of the app needs to know about layer indices
- Can't easily embed custom rendering within a specific widget
- Less compositional - layers are separate from the UI tree
- Doesn't scale well for applications with many custom rendering contexts

### Approach 2: Nested Composition

An alternative is treating custom rendering as just another node in the tree:

```rust
div()
  .child(text("Hello"))
  .child(canvas(|device, queue, view| {
      // Custom shader rendering here
  }))
  .child(button("Click me"))
```

**How it works:** Custom rendering surfaces (canvas, viewport, etc.) are nodes in the retained tree, just like rectangles or text. They compose with everything else.

**Advantages:**
- Fits naturally with entity-map architecture
- Custom rendering can be embedded anywhere
- Z-ordering determined by tree structure
- Scoped - each widget can have its own rendering context
- No global state or coordination needed
- Scales to complex applications (multiple 3D viewports, image thumbnails with effects, etc.)

**Tradeoffs:**
- More complex to implement (requires render-to-texture)
- Needs careful clipping and masking
- Slight overhead from additional render passes

**Performance note:** While render-to-texture adds overhead, it's minimal for UI workloads (~1-2% typically). Modern GPUs handle this efficiently - it's the same technique used by web browsers for canvas elements and by compositor-based window systems. The real performance bottlenecks in UI rendering are draw call counts, shader complexity, and overdraw, not compositing overhead. With optimization strategies like texture atlasing, lazy updates, and culling, hitting 120fps @ 1440p with 1k nodes is entirely feasible.

**Implementation: How nested composition actually works**

When you write `canvas(|device, queue, bounds| { render_3d_scene(); })`, here's what happens under the hood:

1. **Allocate off-screen framebuffer** - Create a GPU texture matching the canvas bounds (e.g., 200×200 pixels)
2. **Render to texture** - Execute user's custom GPU code, rendering to that texture instead of the main framebuffer
3. **Generate bitmap** - The texture now contains the rendered result (a 3D scene, shader effect, whatever)
4. **Composite into UI** - Pass the texture to Vello, which treats it like any other image primitive and composites it at the correct position in the UI tree

This is exactly how web browsers implement `<canvas>` elements - off-screen framebuffer → bitmap → composite into layout.

**Alternative approaches considered:**

**1. Direct rendering with scissor/stencil (rejected)**
- Render custom GPU work directly to main framebuffer
- Use GPU scissor rectangles or stencil buffers to clip to canvas bounds
- **Problem**: Can't handle overlapping UI. If you render a canvas, then need to render UI on top, you'd need to carefully order all draw calls. Breaks the tree abstraction.
- **Problem**: Can't apply transforms (rotation, scale, opacity) to the entire canvas as one unit
- **Problem**: Complex clipping (border-radius, masks, nested transforms) becomes very difficult

**2. Command buffer batching (rejected)**
- Collect all rendering commands (both Vello and custom) in a command buffer
- Sort by z-order and execute in one pass
- **Problem**: Vello and custom GPU work use different pipelines, shaders, and render state. Interleaving them efficiently is complex.
- **Problem**: Still need render-to-texture for transforms, clipping, and effects

**3. Render-to-texture (chosen)**
- Standard GPU pattern, well-optimized by drivers
- Clean abstraction - canvas content is isolated
- Natural support for clipping, transforms, caching
- Same approach used by all major UI frameworks (web browsers, game engines, compositors)

**Why render-to-texture wins:**

The GPU cost of an extra texture is negligible compared to the flexibility gained. Modern GPUs can composite hundreds of textures per frame at 4K resolution. The real bottlenecks are:
- Draw call count (minimized by batching)
- Shader complexity (user-controlled)
- Overdraw (minimized by culling)
- Memory bandwidth (texture compression helps)

The render-to-texture approach gives us proper compositing, which is essential for the "nested in UI tree" property that makes Felt's canvas powerful.

### Chosen Direction: Hybrid Approach

For Felt UI, **we'll support both approaches** - they solve different problems and compose well together.

**Global layers for full-screen concerns:**
```rust
// Define your own layer ordering
enum AppLayer {
    Background,
    PostProcess,
}

impl IntoLayer for AppLayer {
    fn layer_order(&self) -> u32 {
        match self {
            AppLayer::Background => 0,
            AppLayer::PostProcess => 999,
        }
    }
}

// Use semantic layers in your render method
cx.layer(AppLayer::Background, |device, queue| {
    render_animated_starfield(device, queue);
});

cx.layer(AppLayer::PostProcess, |device, queue, scene_texture| {
    apply_bloom(device, queue, scene_texture);
    apply_color_grading(device, queue, scene_texture);
});
```

**Nested composition for widget-level rendering:**
```rust
// Custom rendering embedded in UI tree
div()
  .child(canvas(|device, queue, bounds| {
      render_3d_viewport(device, queue, bounds);
  }))
  .child(sidebar()
    .child(canvas(|device, queue, bounds| { render_thumbnail_1(device, queue, bounds); }))
    .child(canvas(|device, queue, bounds| { render_thumbnail_2(device, queue, bounds); }))
  )
```

**Why both?**

**1. Different use cases**
- Global layers: Full-screen backgrounds, post-processing, screen-space effects
- Nested: 3D viewports, image editors, custom widgets, localized rendering

**2. They compose naturally**
Global post-processing can affect nested canvases. Background layers sit behind the entire UI tree. They work together, not against each other.

**3. Right tool for the job**
Need a full-screen bloom effect? Global layer. Need a custom 3D viewport widget? Nested canvas. Users choose based on their needs.

**4. True flexibility**
An image editor might use both:
```rust
enum EditorLayer {
    Overlay,
}

impl IntoLayer for EditorLayer {
    fn layer_order(&self) -> u32 {
        match self {
            EditorLayer::Overlay => 200,
        }
    }
}

// Global: UI blur when modal is open
if modal_open {
    cx.layer(EditorLayer::Overlay, |device, queue, scene| {
        apply_gaussian_blur(device, queue, scene);
    });
}

// Nested: Image editor with thumbnails
div()
  .child(canvas(|device, queue, bounds| {
      // Main image with custom filter shaders
      render_image_with_filters(device, queue, bounds);
  }))
  .child(sidebar()
    .child(canvas(|device, queue, bounds| {
        render_thumbnail_with_effect(device, queue, bounds);
    }))
  )
```

**5. Matches the philosophy**
"Capability over convenience" means giving users multiple tools that compose. Global layers and nested composition serve different purposes and can work together. More API surface, but more power.

This hybrid approach delivers on the promise of "powerful building blocks" - users get the right primitive for each situation, and they compose naturally.

### Layer Ordering: Type-Safe and Extensible

Raw numeric layer ordering (`cx.layer(0, ...)`, `cx.layer(100, ...)`) is error-prone and not self-documenting. What if you accidentally use `100` and collide with the default UI layer? What does `200` mean in your application's context?

**Solution: The `IntoLayer` trait**

Felt UI uses a trait-based approach that allows both framework-provided semantic layers and user-defined application-specific layer enums:

```rust
/// Trait for types that can be converted to a layer order
pub trait IntoLayer {
    fn layer_order(&self) -> u32;
}

// Accept raw numbers for prototyping
impl IntoLayer for u32 {
    fn layer_order(&self) -> u32 { *self }
}

// Framework-provided minimal layer
pub enum Layer {
    UI,  // The default layer where UI tree renders (100)
}

impl IntoLayer for Layer {
    fn layer_order(&self) -> u32 {
        match self {
            Layer::UI => 100,
        }
    }
}

// Context accepts anything that implements IntoLayer
impl Context {
    pub fn layer<L: IntoLayer>(&mut self, layer: L, callback: ...) -> EntityId {
        let order = layer.layer_order();
        // ...
    }
}
```

**Users define their own semantic layer enums:**

```rust
// Application-specific layer ordering
enum MapLayer {
    Terrain,
    Roads,
    Buildings,
    Weather,
    Overlay,
    PostProcess,
}

impl IntoLayer for MapLayer {
    fn layer_order(&self) -> u32 {
        match self {
            MapLayer::Terrain => 0,
            MapLayer::Roads => 10,
            MapLayer::Buildings => 20,
            MapLayer::Weather => 30,
            MapLayer::Overlay => 200,
            MapLayer::PostProcess => 999,
        }
    }
}

// Usage is clean and semantic:
cx.layer(MapLayer::Terrain, |device, queue| { render_terrain(); });
cx.layer(MapLayer::Roads, |device, queue| { render_roads(); });
cx.layer(MapLayer::PostProcess, |device, queue, scene| { apply_vignette(scene); });

// Framework layer for UI tree
cx.layer(Layer::UI, |device, queue| { /* UI elements render here by default */ });
```

**Benefits:**
- **Type-safe** - Can't accidentally collide with magic numbers
- **Self-documenting** - `MapLayer::Terrain` is clearer than `0`
- **Domain-specific** - Each app defines layer names that match its domain
- **Minimal framework** - Felt only provides Layer::UI, users define the rest
- **Backward compatible** - Raw `u32` still works for quick prototyping

**Why only Layer::UI?**

Felt provides only one layer enum variant: `Layer::UI` (at order 100). Concepts like "Background," "Overlay," or "PostProcess" are application-specific, not framework concerns. A mapping app has different layer semantics than an image editor or audio visualizer.

By providing only the trait mechanism and the default UI layer, Felt trusts developers to define semantic layers that match their domain. This matches the philosophy: provide powerful primitives (IntoLayer trait), not prescriptive abstractions.

### Concrete Example: Map Application with HUD and Modal

Here's how a mapping application would use the hybrid approach - combining custom map rendering, nested canvas for a minimap HUD, UI overlays, blur effects, and modals. This example follows GPUI's structure with Context as the central API:

```rust
use felt::{
    div, canvas, prelude::*, Application, Context, IntoLayer, Render, Window, WindowOptions,
};

// Application-specific layer ordering
enum MapLayer {
    Background,   // 0   - Full-screen map rendering
    Overlay,      // 200 - Blur effects
    ModalUI,      // 300 - Modal UI above blur
    PostProcess,  // 999 - Final post-processing
}

impl IntoLayer for MapLayer {
    fn layer_order(&self) -> u32 {
        match self {
            MapLayer::Background => 0,
            MapLayer::Overlay => 200,
            MapLayer::ModalUI => 300,
            MapLayer::PostProcess => 999,
        }
    }
}

struct MapApp {
    modal_open: bool,
    // Store layer ID for dynamic management
    blur_layer: Option<EntityId>,
}

impl Render for MapApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Global layer: Main map background (using app-specific MapLayer enum)
        cx.layer(MapLayer::Background, |device, queue| {
            // Custom GPU rendering: terrain, roads, buildings
            render_map_with_shaders(device, queue);
        })
        .id("map-background");

        // Global layer: Blur effect - dynamically managed
        if self.modal_open && self.blur_layer.is_none() {
            // Create blur layer when modal opens
            self.blur_layer = Some(
                cx.layer(MapLayer::Overlay, |device, queue, scene| {
                    apply_gaussian_blur(device, queue, scene, 10.0);
                })
                .id("modal-blur")  // Optional id
            );
        } else if !self.modal_open && self.blur_layer.is_some() {
            // Remove blur layer when modal closes
            cx.remove_layer(self.blur_layer.unwrap());
            self.blur_layer = None;
        }

        // Global layer: Final post-processing
        cx.layer(MapLayer::PostProcess, |device, queue, scene| {
            apply_vignette(device, queue, scene);
        })
        .id("post-fx");

        // Build UI tree (renders at default layer 100)
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // Top toolbar
                div()
                    .flex()
                    .gap_2()
                    .p_4()
                    .child(button("Zoom In"))
                    .child(button("Zoom Out"))
                    .child(search_box())
            )
            .child(
                // Mini-map HUD with nested canvas element
                div()
                    .absolute()
                    .top_4()
                    .right_4()
                    .size(px(200.0))
                    .bg(rgba(0x000000, 0.7))
                    .border_2()
                    .border_color(rgb(0x00ff00))
                    .p_2()
                    .child(
                        // Nested canvas: custom GPU rendering within UI tree
                        canvas(|device, queue, bounds| {
                            // Custom shader renders minimap
                            render_minimap_shader(device, queue, bounds);
                            // Draw player position indicator
                            render_player_marker(device, queue);
                        })
                        .size_full()
                    )
                    .child(
                        // Vello-rendered text overlay
                        text("Mini-Map")
                            .text_xs()
                            .text_color(rgb(0xffffff))
                    )
            )
            .child(
                // Info panel (bottom right)
                div()
                    .absolute()
                    .bottom_4()
                    .right_4()
                    .bg(rgba(0x000000, 0.8))
                    .p_4()
                    .border_1()
                    .border_color(rgb(0xffffff))
                    .child(text("Current Location: ..."))
                    .child(text("Coordinates: ..."))
            )
            .child_if(self.modal_open,
                // Modal dialog - renders at MapLayer::ModalUI (above blur)
                modal_dialog()
                    .layer(MapLayer::ModalUI)  // Render this subtree at layer 300
                    .child(text("Share Location?"))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(button("Cancel"))
                            .child(button("Share"))
                    )
            )
    }
}

fn main() {
    Application::new().run(|cx| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(MapApp {
                modal_open: false,
                blur_layer: None,
            })
        });
    });
}
```

**Visual Stack (back to front):**

```
┌─────────────────────────────────────┐
│  MapLayer::PostProcess: Vignette    │ ← Post-processing (999)
├─────────────────────────────────────┤
│  MapLayer::ModalUI: Modal Dialog    │ ← Modal UI above blur (300)
│  ┌─────────────────────────────┐    │
│  │ Share Location?             │    │
│  │ [Cancel]  [Share]           │    │
│  └─────────────────────────────┘    │
├─────────────────────────────────────┤
│  MapLayer::Overlay: Blur (if modal) │ ← Blurs layers 0-100 (200)
├─────────────────────────────────────┤
│  Layer::UI: Main UI Tree            │ ← Default Felt layer (100)
│  ┌───────────────────────────────┐  │
│  │ [Zoom +] [Zoom -] [🔍______] │  │ ← Toolbar (Vello)
│  └───────────────────────────────┘  │
│                                      │
│              ┌──────────┐            │
│              │╔════════╗│            │ ← HUD (nested canvas)
│              │║ Canvas ║│            │   Custom GPU rendering
│              │║ Minimap║│            │   within UI tree
│              │║   🔴   ║│            │
│              │╚════════╝│            │
│              │Mini-Map  │            │ ← Vello text overlay
│              └──────────┘            │
│                                      │
│                  ┌────────────┐      │
│                  │Location:...│      │ ← Info panel (Vello)
│                  │Coords: ... │      │
│                  └────────────┘      │
├─────────────────────────────────────┤
│  MapLayer::Background: Main Map     │ ← Custom GPU rendering (0)
│  (terrain, roads, buildings)        │   Full-screen background
└─────────────────────────────────────┘
```

**What's happening:**

1. **MapLayer::Background (Main Map)** - Full-screen custom GPU rendering for terrain, roads, buildings. This is the background for everything.

1. **Layer::UI (Main UI Tree)** - Contains all standard UI elements built with Vello (Felt's default layer):
   - **Toolbar** - Buttons and search box (Vello primitives)
   - **HUD with nested canvas** - The minimap uses `canvas()` element for custom GPU rendering (shader-based), wrapped in Vello UI (border, background, text). This shows nested composition in action.
   - **Info panel** - Location display (Vello text and containers)

1. **MapLayer::Overlay (Blur)** - Conditional full-screen blur effect, only active when modal is open. Blurs everything below it (layers 0-100).

1. **MapLayer::ModalUI (Modal Dialog)** - The modal is rendered at a user-defined layer (300) above the blur, so it stays sharp while the background is blurred. This shows `.layer()` method usage: the modal subtree is rendered at a different layer than the rest of the UI.

1. **MapLayer::PostProcess (Vignette)** - Post-processing on the final composition.

**Key insights:**

- **Global layers** for full-screen concerns (map background, blur, vignette)
- **Nested canvas** for widget-level custom rendering (minimap HUD with shaders)
- **Vello UI** for all standard elements (buttons, text, containers, positioning)
- **Context as central API** - `cx.layer()` for global, `canvas()` for nested
- **GPUI-style structure** - `Render` trait, state in `self`, framework services in `cx`
- **User-defined layer enums** - MapLayer is app-specific, not provided by Felt. Each app defines semantic layers that match its domain (Background, Overlay, ModalUI, PostProcess, etc.)
- **Subtree layer override** - The `.layer()` method allows rendering part of the UI tree at a different layer (e.g., modal at MapLayer::ModalUI above blur at MapLayer::Overlay)
- **Only one framework UI layer** - Felt provides only `Layer::UI`. Users create additional UI layers (like MapLayer::ModalUI) when needed for their specific use case
- **Layer management** - Layers return `EntityId` for dynamic control, with optional `.id()` for naming (web-familiar). Store EntityIds in state to add/remove layers conditionally (like the blur layer that only exists when modal is open)

The minimap HUD perfectly demonstrates the hybrid approach: it uses a `canvas()` element (custom GPU shaders) but is positioned, styled, and bordered using regular UI primitives. It's part of the UI tree, not a separate rendering system.

The blur layer demonstrates entity-based layer management: it's created dynamically when needed, stored as an `EntityId`, and removed when no longer required. This fits naturally with the entity-map architecture where layers are entities like any other UI element.

---

## Why This Matters: Solving the Creative Tools Problem

Remember the problem from the beginning: most UI frameworks force you to choose between great UI or GPU access. You can't have both.

**The map example above shows why Felt is different:**

**Custom GPU rendering for the map terrain** - Full shader control for rendering terrain, roads, and buildings. No framework abstractions limiting what you can do. This is the "GPU freedom" part that frameworks like gpui don't provide.

**Pixel-perfect UI with Vello** - Buttons, text, panels, and modals rendered with high-quality 2D primitives. Crisp text via Parley, vector graphics, proper layout. This is the "great UI" part that low-level graphics APIs don't provide.

**Nested canvas for the minimap** - Custom GPU shaders embedded directly in the UI tree, positioned with regular layout primitives. You couldn't do this with traditional UI frameworks - they hide the GPU. You couldn't do this with raw wgpu - no UI primitives.

**Dynamic post-processing layers** - Full-screen blur effect that only exists when the modal is open. Entity-based lifecycle, shader-based effect, composed with the entire UI. This level of composability between UI and GPU rendering is what makes creative tools feel polished.

**This is the UI framework for creative tools.** Image editors that need real-time shader-based filters AND pixel-perfect UI. 3D modeling tools that need custom render pipelines AND professional toolbars. Audio tools that need GPU-accelerated waveform rendering AND interactive controls. Data visualizations that need compute shaders AND responsive layouts.

Felt doesn't force you to choose. You get both. That's why it exists.
