//! # AeroDB Real-Time Module
//!
//! Phase 10: Real-Time Subscriptions
//!
//! This module provides real-time event subscriptions over WebSocket,
//! with deterministic event generation and best-effort delivery.
//!
//! ## Architecture
//!
//! - **Event Log** (deterministic): WAL → Event transformation
//! - **Dispatcher** (non-deterministic): WebSocket event delivery
//! - **Subscriptions**: Client subscription management
//! - **Broadcast**: Pub/sub channels
//! - **Presence**: User presence tracking

pub mod errors;
pub mod event;
pub mod event_log;
pub mod subscription;
pub mod dispatcher;
pub mod broadcast;
pub mod presence;

pub use errors::{RealtimeError, RealtimeResult};
pub use event::{DatabaseEvent, EventType, BroadcastEvent};
pub use event_log::EventLog;
pub use subscription::{Subscription, SubscriptionRegistry};
pub use dispatcher::Dispatcher;
pub use broadcast::BroadcastChannel;
pub use presence::PresenceTracker;
