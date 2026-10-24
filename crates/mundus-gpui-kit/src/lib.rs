//! mundus-gpui-kit — shared GPUI surface extracted from `cortex/manager-gpui`:
//! Imago theme tokens (`theme`), shell chrome (`widgets`), view primitives and
//! JSON accessors (`fields`), and the Engine lock/RPC client (`engine`).
//!
//! Consumed via pinned git dependency by manager-gpui, the unified Cortex GPUI
//! application and agenda-gpui. The `gpui`/`gpui-component`/`imago-gpui`
//! dependency specs must match every consumer exactly.

pub mod engine;
pub mod engine_error;
pub mod engine_ws;
pub mod fields;
pub mod theme;
pub mod widgets;
