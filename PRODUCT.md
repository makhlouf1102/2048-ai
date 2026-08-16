# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Casual 2048 players who want to play the puzzle themselves or watch and understand an AI player in action.

## Product Purpose

An approachable, responsive 2048 game where the same board can be controlled by a person or by an AI player named Milo. Success means the game feels delightful to play, makes the AI's activity legible, and works comfortably with keyboard, WASD, touch, and programmatic input.

## Positioning

The playable board and the observable AI share one live game session: Milo's moves, decision timing, and state changes are reflected directly in the same interface the player can control.

## Operating Context

Players may play manually, hand control to Milo, pause the AI, restart, continue after winning, or save a screenshot after losing. The interface is used on desktop and mobile web.

## Capabilities and Constraints

- Preserve the underlying 2048 rules, game state, manual controls, AI autoplay contract, and responsive behavior.
- Milo remains the named AI character, but his visual treatment, voice presentation, telemetry, labels, hierarchy, and surrounding UI may be redesigned.
- Existing secondary copy and information architecture may be modified where doing so improves clarity and coherence.
- The Rust/WebAssembly AI and browser game API remain implementation constraints.

## Brand Commitments

- Milo is the AI player's name and recognizable character.
- The replacement identity uses a colorful, childlike, hand-made geometric language inspired by Paul Klee's playful compositions.
- The experience should feel artful and imaginative without becoming difficult to operate.

## Evidence on Hand

- Existing responsive game and AI monitor implementation in `app/features/game/components/Game.tsx` and `app/app.css`.
- Existing Milo avatar at `public/assets/milo-avatar.png`.
- No testimonials, commercial claims, or external proof assets are required or available.

## Product Principles

- Keep the puzzle immediately playable.
- Make Milo feel like a companion with an observable thought process, not a detached analytics panel.
- Let expressive art direction support state recognition and hierarchy.
- Preserve direct, accessible input across keyboard and touch.

## Accessibility & Inclusion

Maintain semantic controls, visible focus states, readable contrast, reduced-motion support, and non-color-only state cues.
