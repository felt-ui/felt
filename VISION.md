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
