# Felt UI Roadmap
Feasible. Three-year solo roadmap to a credible, production-ready UI library.

## Scope

A GPU-accelerated Desktop UI engine. Targets macOS, Windows, Wayland.

## Stack
 - winit
 - wgpu
 - vello
 - taffy
 - parley
 - accesskit
 - muda

## Non-negotiables

- Zero undefined behavior.
- Smooth at 120 Hz at 1440p for 1k nodes.
- Deterministic input, IME, focus.
- A11y hooks present, even if partial.

## Milestones

### M0–M3: Core loop and rectangles

- Windowing, swapchain, DPI.
- Retained tree with EntityId and EntityMap.
- Taffy layout bridge.
- Vello scene build, clip, transforms.
- Input routing, hit-test, hover, press.

**Demo:** panel layout + clickable buttons.
**Perf goal:** 200k rects static at 60 Hz.
**Exit tests:** resize, multi-monitor DPI, vsync modes.

### M4–M6: Text

- Parley integration, shaping cache, caret, selection.
- Clipboard, basic IME.
- Text measurement API.

**Demo:** editable TextField and multiline editor.
**Perf goal:** 10k glyphs dynamic at 120 Hz.
**Exit tests:** CJK, RTL, emoji, dead keys.

### M7–M9: Images and scroll

- Image loading, atlases, nine-patch.
- ScrollView with inertia.
- Layer system. Overlay and tooltip.

**Demo:** list with thumbnails and text.
**Perf goal:** 10k items virtualized at 120 Hz.

### M10–M12: Widgets v1 and theming

- Button, Checkbox, Slider, TextField, List, Split, Tabs.
- Theming tokens and CSS-like styles.
- Minimal animation system.

**Demo:** component gallery app.
**Perf goal:** gallery at 120 Hz on M1 Pro and RTX 3060.

### M13–M18: Persistence and editor canvas

- Command stack, undo/redo.
- Vector canvas with selection and transforms.
- Zoom and pan.

**Demo:** simple 2D editor drawing rectangles with handles.
**Perf goal:** 5k shapes interactive at 120 Hz.

### M19–M24: A11y and menus

- accesskit wiring.
- Native menus via muda.
- Context menus, accelerators, focus ring.

**Demo:** keyboard-only navigation across gallery.
**Exit tests:** VoiceOver, Narrator, Orca smoke tests.

### M25–M30: Stability push

- Fuzz input. Property tests for layout and hit-test.
- Crash-free telemetry harness.
- API polish and docs site with live examples.

**Exit:** v0.9 tagged. Used in one real app.

### M31–M36: v1

- API freeze review, audits, benchmarks.
- LTS branch.
- Publish v1.0.0.
