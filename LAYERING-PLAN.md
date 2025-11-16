# Layering Architecture: Implementation Plan

## Timeline

```
Current Phase          Next Phase              Future Phase
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   Vello      │  →   │  Plan        │  →   │  Retained    │
│ Integration  │      │  Layering    │      │  Tree        │
└──────────────┘      └──────────────┘      └──────────────┘
  (Now)                (After Vello)         (M0-M3 Step 5)
```

**Key insight:** Plan layering architecture after basic Vello works, but before designing the retained tree.

## Current Phase: Vello Integration

**Goal:** Get rectangles rendering with Vello

**Tasks:**
- [ ] Add vello dependency
- [ ] Create vello::Renderer and integrate with wgpu
- [ ] Build vello::Scene with rectangle primitives
- [ ] Render scene to surface texture
- [ ] Handle scene rebuilding on resize
- [ ] Benchmark: 200k static rects at 60 Hz

**Don't worry about layers yet.** Focus on understanding Vello's Scene API and getting the basic render loop working.

## Next Phase: Plan Layering Architecture

**Trigger:** Once you can render basic Vello scenes (rectangles working, 60 Hz achieved)

**Duration:** 1-2 hours of design work, document findings

### Questions to Answer

#### 1. Frame Render Sequence

What order do things render in?

```
Layer 0 (Background)
  ↓
Layer 100 (UI/Vello)
  ↓
Layer 200 (Overlay/Effects)
  ↓
Layer 999 (Post-processing)
```

**Design questions:**
- How do you track which layers exist?
- How do you sort layers by order?
- Can layers be added/removed dynamically?

#### 2. Vello's Role in Layering

**Key question:** Is Vello always at Layer::UI (100)? Or can multiple Vello scenes render at different layers?

**Option A: Single Vello scene at Layer::UI**
```rust
// All UI renders to one Vello scene at layer 100
cx.layer(Layer::UI, |device, queue| {
    // Vello scene rendered here
});
```

**Option B: Multiple Vello scenes at different layers**
```rust
// Modal UI gets its own Vello scene at layer 300
modal_dialog()
    .layer(MapLayer::ModalUI)  // Separate Vello scene?
    .child(text("Modal"))
```

**Implications:**
- Option A: Simpler, one scene rebuild
- Option B: More flexible, but multiple scene rebuilds

**Decision needed:** Which approach?

#### 3. Global Layer API

How do users register global layers for custom GPU work?

```rust
cx.layer(MapLayer::Background, |device, queue| {
    // Custom GPU rendering
    render_map_with_shaders(device, queue);
})
```

**What does the renderer need to support this?**
- Callback storage (closures with wgpu::Device, wgpu::Queue access)
- Render pass management (when to create render passes)
- Intermediate textures (for blending layers)

**Design questions:**
- Where are layer callbacks stored? (In Application? In Renderer?)
- How do callbacks get wgpu::Device and wgpu::Queue?
- How do post-processing layers get the scene texture?
  ```rust
  cx.layer(MapLayer::Overlay, |device, queue, scene_texture| {
      apply_blur(device, queue, scene_texture);
  })
  ```

#### 4. Canvas Implementation Strategy

Render-to-texture for nested canvas requires:

```rust
canvas(|device, queue, bounds| {
    render_minimap_shader(device, queue, bounds);
})
```

**Implementation needs:**
1. Off-screen framebuffer allocation (wgpu::Texture)
2. Execute user's custom GPU code to that texture
3. Convert texture to Vello image primitive
4. Composite into scene at correct position

**Design questions:**
- When is the texture allocated? (On first render? Pre-allocated pool?)
- When is it re-rendered? (Every frame? Only on invalidation?)
- How does it integrate with Vello's Scene API?
- Where does the texture → Vello image conversion happen?

**Note:** Full implementation is probably M4+, but plan the architecture now.

## Why Plan Before Retained Tree?

The retained tree design depends on understanding layering:

**Without layering plan:**
```rust
struct Node {
    // What goes here? Do we need layer info?
    // Are canvas nodes special?
    // How do subtrees override layers?
}
```

**With layering plan:**
```rust
struct Node {
    layer_override: Option<LayerOrder>,  // For .layer() API
    // Clear understanding of how this affects rendering
}

// Canvas is just another node type, not special
enum NodeType {
    Div,
    Text,
    Canvas(CanvasRenderer),  // Render-to-texture handled by renderer
}
```

**Questions the retained tree design needs answered:**
1. Does the tree store layer information per-node?
2. Are canvas nodes a special type, or just another element?
3. How does tree traversal interact with layer rendering?
4. Does the tree need to be traversed multiple times (once per layer)?

## Recommended Approach

### Step 1: Get Vello working (Current)
- Basic rectangles rendering
- 60 Hz performance
- Resize handling

### Step 2: Design layering architecture (Next - 1-2 hours)
Create a design document answering:
1. Frame render sequence (how layers are sorted and executed)
2. Vello's role (single scene vs multiple scenes)
3. Global layer API design (callback storage, render pass management)
4. Canvas implementation approach (texture allocation, Vello integration)
5. How retained tree will interact with layers

**Output:** `LAYERING-DESIGN.md` with API signatures and architecture decisions

### Step 3: Design retained tree (M0-M3 Step 5)
With layering architecture understood:
- Design Node structure
- Tree traversal strategy
- Entity-map integration
- How nodes specify layer overrides

### Step 4: Implement basic global layers (M4)
- Layer registration API
- Render pass management
- Simple custom GPU rendering

### Step 5: Implement canvas (M4+)
- Render-to-texture
- Vello image integration
- Full nested composition

## Success Criteria

Before moving to retained tree design, you should be able to answer:

- ✅ How do layers render in sequence?
- ✅ Where does Vello fit in the layer system?
- ✅ What API do users call to register global layers?
- ✅ How does the renderer manage layer callbacks and render passes?
- ✅ How will canvas render-to-texture work (high-level)?
- ✅ What does the retained tree need to store to support layers?

## Next Actions

1. **Now:** Finish Vello integration (M0-M3 Step 4)
2. **After Vello works:** Create `LAYERING-DESIGN.md` answering the questions above
3. **Then:** Design retained tree with layering in mind (M0-M3 Step 5)
