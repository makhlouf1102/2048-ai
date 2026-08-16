---
name: "2048 AI — Milo's Painted Playroom"
description: "A hand-painted number town where people and Milo share one playful 2048 board."
colors:
  parchment: "#f5e8c8"
  parchment-light: "#fff8e8"
  ink: "#20261f"
  ink-soft: "#4c5548"
  vermilion: "#e9573f"
  cobalt: "#3367b1"
  marigold: "#f4b93a"
  leaf: "#5b9b5a"
  lilac: "#a77ab8"
  terracotta: "#bd6949"
typography:
  display:
    fontFamily: "Bowlby One, Arial Rounded MT Bold, sans-serif"
    fontSize: "clamp(64px, 9vw, 114px)"
    fontWeight: 400
    lineHeight: 0.78
    letterSpacing: "-0.03em"
  headline:
    fontFamily: "Bowlby One, Arial Rounded MT Bold, sans-serif"
    fontSize: "clamp(36px, 4vw, 54px)"
    fontWeight: 400
    lineHeight: 0.9
    letterSpacing: "-0.02em"
  title:
    fontFamily: "Bowlby One, Arial Rounded MT Bold, sans-serif"
    fontSize: "clamp(19px, 2.4vw, 28px)"
    fontWeight: 400
    lineHeight: 1.05
  body:
    fontFamily: "Nunito Sans, ui-sans-serif, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.45
  label:
    fontFamily: "Nunito Sans, ui-sans-serif, sans-serif"
    fontSize: "10px"
    fontWeight: 900
    lineHeight: 1
    letterSpacing: "0.09em"
rounded:
  key: "5px 7px 4px 6px"
  control: "12px 14px 11px 15px"
  tile: "10px 15px 11px 14px"
  board: "18px 23px 20px 16px"
  studio: "21px 17px 25px 19px"
spacing:
  xs: "8px"
  sm: "12px"
  md: "18px"
  lg: "28px"
  xl: "clamp(28px, 4vw, 64px)"
components:
  button-milo:
    backgroundColor: "{colors.leaf}"
    textColor: "{colors.parchment-light}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "11px 17px"
    height: "46px"
  button-new-board:
    backgroundColor: "{colors.marigold}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "11px 17px"
    height: "46px"
  button-secondary:
    backgroundColor: "{colors.parchment-light}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "11px 17px"
    height: "46px"
  score-card:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.parchment-light}"
    rounded: "{rounded.control}"
    padding: "10px 14px 9px"
  game-tile:
    backgroundColor: "{colors.marigold}"
    textColor: "{colors.ink}"
    typography: "{typography.display}"
    rounded: "{rounded.tile}"
---

# Design System: 2048 AI — Milo's Painted Playroom

## Overview

**Creative North Star: "Milo's Painted Playroom"**

The 2048 board is a painted little town that Milo learns to navigate. Broad gouache fields, warm paper, fine ink outlines, and deliberately imperfect geometry make the game feel assembled by hand while clear labels and stable placement keep every action immediately operable.

This world is colorful, companionable, tactile, and gently mischievous. Expression comes from pigment, irregular silhouettes, small rotations, and stateful motion—not from ornamental dashboard chrome. It explicitly refuses the beige utility-dashboard language common to puzzle and AI interfaces.

**Key Characteristics:**

- Warm parchment beneath decisive vermilion, cobalt, marigold, leaf, and lilac fields.
- Fine near-black outlines that keep painted surfaces legible and operational.
- Chunky display numerals paired with friendly, high-weight utility text.
- Slightly skewed controls and asymmetric corners, always within a stable layout.
- Milo presented as a companion inside the same game world, not as a detached analytics panel.

## Colors

The palette behaves like a compact gouache box: saturated colors carry identity and state, while parchment and ink provide continuity and contrast.

### Primary

- **Vermilion:** The sharpest accent for the first half of the wordmark, active or interrupted AI states, and small moments that need immediate attention.
- **Cobalt:** The anchoring cool accent for the second half of the wordmark, secondary score surfaces, selection, and the universal focus outline.

### Secondary

- **Marigold:** The optimistic action color for new-board controls, directional energy, and high-value tiles.
- **Leaf:** Milo's ready and active companion color, used on the primary handoff action and live status cues.

### Tertiary

- **Lilac:** A supporting paint field for telemetry and high-number tile variation.
- **Terracotta:** A grounded warm pigment reserved for secondary painted fields and material variation.

### Neutral

- **Parchment:** The main page ground and translucent overlay base; it should remain visibly warm.
- **Light Parchment:** The brightest readable surface for control text, studio panels, and inset details.
- **Ink:** The structural color for text, outlines, board framing, and the darkest score surface.
- **Soft Ink:** Supporting copy and low-priority status text; never use it where stronger contrast is required.

### Named Rules

**The Painted Field Rule.** Saturated colors appear as broad, bounded fields with ink edges; do not scatter them as tiny dashboard-status accents.

**The Cobalt Focus Rule.** Every interactive control uses the same unmistakable cobalt focus outline, regardless of its resting paint color.

## Typography

**Display Font:** Bowlby One (with Arial Rounded MT Bold and sans-serif fallbacks)  
**Body Font:** Nunito Sans (with ui-sans-serif and sans-serif fallbacks)

**Character:** Bowlby One makes numbers and names feel cut from painted card, while Nunito Sans keeps instructions, states, and telemetry soft, compact, and highly readable. Weight supplies authority; excessive type decoration does not.

### Hierarchy

- **Display** (400, fluid oversized, tight line height): Reserved for the 2048 wordmark and the board's numerals.
- **Headline** (400, fluid large, compact line height): Milo's name and game-result messages.
- **Title** (400, fluid medium, compact line height): Direction decisions and other signature readouts.
- **Body** (700–800, 13–15px, relaxed line height): Instructions, thoughts, statuses, and supporting explanation.
- **Label** (900, 9–11px, tracked uppercase): Scores, metric names, and compact state labels.

### Named Rules

**The Chunk and Whisper Rule.** Use Bowlby One for the few facts that should feel monumental; let Nunito Sans carry all explanation and operation.

## Layout

Desktop uses a centered two-part composition up to 1320px wide: the playable town occupies the larger left column and Milo's narrower studio occupies the right, with a fluid 28–64px gap. The complete board and studio should share the first viewport at the intended desktop review size.

Below 1050px, the composition becomes one centered column up to 680px wide. The complete board comes first and Milo follows immediately; the studio stops being sticky. Below 620px, outer padding tightens, the header and toolbar stack, actions share the available width, board gaps contract, and Milo's portrait and decision mark scale down without changing the reading order. Spacing follows a compact 8/12/18/28 rhythm, opening up only between the two primary desktop regions.

**The Shared-Board Rule.** Scores, controls, board, and Milo describe one live session; never separate Milo into a remote dashboard or a competing page region.

## Elevation & Depth

Depth is physical and shallow: dark, soft-edged shadows suggest thick painted paper lifted from the parchment. The board and Milo's studio carry the strongest resting depth; buttons, score cards, the avatar, and the direction mark carry smaller tactile shadows. Tonal fields and ink borders remain more important than elevation.

### Shadow Vocabulary

- **Control lift** (`3px 7px 18px rgba(32, 38, 31, .16)`): Resting buttons and other small interactive paper pieces.
- **Board lift** (`9px 18px 35px rgba(32, 38, 31, .22)`): The main playable object.
- **Studio lift** (`10px 20px 40px rgba(32, 38, 31, .2)`): Milo's companion panel.
- **Inset pigment edge** (`inset 0 0 0 2px rgba(255, 248, 232, .2)`): A faint tile highlight that suggests uneven paint coverage.

**The Paper, Not Glass Rule.** Shadows imply layered card and painted board stock; never use blur-heavy glass panels, glossy reflections, or floating SaaS cards.

## Shapes

Forms are compact and outlined with 1.5–3px ink strokes. Corners are softly irregular through unequal radii, and a few major objects rotate by less than four degrees. Circular forms are reserved for status lamps and the studio sun; most of the world remains geometric and hand-cut.

**The Gentle Skew Rule.** Rotation adds handmade character to a control or focal object, but must reset or settle on interaction and must never disturb alignment or hit areas.

## Components

### Buttons

- **Shape:** Tactile painted tabs with asymmetric 12–15px corners, a 2px ink outline, and at least 46px height.
- **Primary:** Milo's handoff action is leaf green with light parchment text and a bordered circular indicator; when running, it turns vermilion and the indicator pulses.
- **Secondary:** New-board actions are marigold; quieter overlay actions use light parchment.
- **Hover / Focus:** Hover lifts 3px and settles the resting rotation. Active state presses down 1px. Keyboard focus is a 4px cobalt outline offset by 4px.

### Cards / Containers

- **Score cards:** Compact dark-ink or cobalt plaques with centered uppercase labels and tabular display numerals.
- **Game board:** The dominant framed object, with a 3px ink border, irregular 16–23px corners, painted-paper texture, a slight counterclockwise rotation, and a fixed 4×4 grid.
- **Milo studio:** A tall light-parchment companion surface with a painted spectrum rail, an avatar field, a thought strip, a decision console, and three compact metric fields.
- **Material:** Generated painted-paper texture may supply pigment variation, but content, numbers, controls, and state remain semantic HTML and CSS.

### Navigation

There is no site navigation on the game surface. Keep focus on the board, its two primary actions, and Milo's live companion state rather than introducing app-shell chrome.

### Painted Tiles

Each tile is a square painted lot with an ink outline, uneven corners, and Bowlby One numerals. Value recognition combines number text with a stable progression of distinct warm and cool paint colors. Empty lots remain visibly darker and quieter than numbered tiles; 2048 receives an additional vermilion-and-marigold inset frame.

### Milo's Decision Mark

The latest direction appears as a large hand-painted, near-circular mark with an inline arrow. Its adjacent text states both the move and decision time, so color and motion are never the sole state cue. Movement uses short, springy reactions; all animation collapses under reduced-motion preferences.

## Do's and Don'ts

### Do:

- **Do** keep the board the largest object and Milo the clear second focal point.
- **Do** use broad gouache fields, fine ink outlines, uneven radii, and small rotations as one coherent material language.
- **Do** preserve semantic controls, visible cobalt focus states, direct keyboard and touch input, and non-color state labels.
- **Do** keep generated raster texture content-free and preserve embedded provenance on every shipping raster.
- **Do** place the complete board before Milo on mobile, with the studio following immediately below.

### Don't:

- **Don't** turn the experience into a beige utility dashboard, analytics console, or grid of interchangeable rounded cards.
- **Don't** use color alone to communicate Milo's state, the latest move, tile value, or game outcome.
- **Don't** add glassmorphism, glossy gradients, generic icon-library ornament, or perfect corporate geometry.
- **Don't** let texture compete with tile numerals, control labels, or telemetry.
- **Don't** expand the palette casually; the compact paint box is part of the identity.
