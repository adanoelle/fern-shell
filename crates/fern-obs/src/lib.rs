//! # fern-obs — OBS Studio WebSocket Bridge
//!
//! `fern-obs` provides a daemon that connects to OBS Studio via the
//! obs-websocket protocol (v5.x), exposing recording and streaming state
//! to the Fern Shell QML interface.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     fern-obs daemon                          │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │              ObsClient (obws)                        │    │
//! │  │  • Maintains WebSocket connection                    │    │
//! │  │  • Subscribes to OBS events                          │    │
//! │  │  • Sends commands (start/stop recording, etc.)       │    │
//! │  └──────────────────────┬──────────────────────────────┘    │
//! │                         │                                    │
//! │  ┌──────────────────────▼──────────────────────────────┐    │
//! │  │              State Manager                           │    │
//! │  │  • Tracks ObsState                                   │    │
//! │  │  • Writes to ~/.local/state/fern/obs-state.json      │    │
//! │  │  • Handles reconnection logic                        │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              ~/.local/state/fern/obs-state.json              │
//! │  {                                                           │
//! │    "connected": true,                                        │
//! │    "recording": { "active": true, "paused": false, ... },    │
//! │    "streaming": { "active": false, ... },                    │
//! │    "current_scene": "Gaming",                                │
//! │    "scenes": ["Gaming", "Desktop", "BRB"],                   │
//! │    ...                                                       │
//! │  }                                                           │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    QML (FileView)                            │
//! │  Obs.qml singleton watches state file, exposes to UI         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Start the daemon (connects to OBS on localhost:4455)
//! fern-obs daemon
//!
//! # With custom host/port/password
//! fern-obs daemon --host localhost --port 4455 --password secret
//!
//! # One-shot commands (require daemon to be running)
//! fern-obs start-recording
//! fern-obs stop-recording
//! fern-obs pause-recording
//! fern-obs scene "Gaming"
//! fern-obs status
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod client;
pub mod config;
pub mod daemon;
pub mod error;
pub mod state;

pub use client::ObsClient;
pub use config::ObsConfig;
pub use error::{Error, Result};
pub use state::{ObsState, RecordingState, StreamingState};
