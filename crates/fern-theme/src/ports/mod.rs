//! # Ports — Hexagonal Architecture Interfaces
//!
//! This module defines the **ports** in fernctl's hexagonal architecture.
//! Ports are trait-based interfaces that decouple the domain logic from
//! external systems, enabling testability and flexibility.
//!
//! ## What are Ports?
//!
//! In hexagonal architecture (also known as "ports and adapters"), ports are
//! the interfaces through which the application core communicates with the
//! outside world. They define *what* operations are available without
//! specifying *how* those operations are implemented.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        EXTERNAL WORLD                               │
//! │   ┌────────────┐    ┌────────────┐    ┌────────────┐               │
//! │   │ TOML Files │    │    CLI     │    │    D-Bus   │               │
//! │   └─────┬──────┘    └─────┬──────┘    └─────┬──────┘               │
//! │         │                 │                 │                       │
//! │   ┌─────▼──────┐    ┌─────▼──────┐    ┌─────▼──────┐               │
//! │   │TomlAdapter │    │ CliAdapter │    │DbusAdapter │  ← ADAPTERS   │
//! │   └─────┬──────┘    └─────┬──────┘    └─────┬──────┘               │
//! └─────────┼─────────────────┼─────────────────┼───────────────────────┘
//!           │                 │                 │
//!     ╔═════╧═════════════════╧═════════════════╧═════╗
//!     ║              INBOUND PORTS                    ║
//!     ║  ConfigPort, ValidatePort, QueryPort          ║  ← INTERFACES
//!     ╚═════════════════════╤═════════════════════════╝
//!                           │
//!     ┌─────────────────────▼─────────────────────────┐
//!     │                 DOMAIN                        │
//!     │        Theme, Tokens, Validation              │  ← PURE LOGIC
//!     └─────────────────────┬─────────────────────────┘
//!                           │
//!     ╔═════════════════════╧═════════════════════════╗
//!     ║             OUTBOUND PORTS                    ║
//!     ║  PersistPort, NotifyPort, IpcPort             ║  ← INTERFACES
//!     ╚═════╤═════════════════╤═════════════════════╤═╝
//!           │                 │                     │
//! ┌─────────┼─────────────────┼─────────────────────┼───────────────────┐
//! │   ┌─────▼──────┐    ┌─────▼──────┐    ┌─────────▼────┐             │
//! │   │FileAdapter │    │NotifAdapter│    │QuickShellIpc │  ← ADAPTERS │
//! │   └─────┬──────┘    └─────┬──────┘    └─────────┬────┘             │
//! │         │                 │                     │                   │
//! │   ┌─────▼──────┐    ┌─────▼──────┐    ┌─────────▼────┐             │
//! │   │ File System│    │ D-Bus Notif│    │   QuickShell │             │
//! │   └────────────┘    └────────────┘    └──────────────┘             │
//! │                        EXTERNAL WORLD                               │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Inbound vs Outbound
//!
//! Ports are divided into two categories based on the direction of dependency:
//!
//! | Category | Direction | Purpose | Examples |
//! |----------|-----------|---------|----------|
//! | **Inbound** | External → Domain | Drive the application | Load config, validate, query |
//! | **Outbound** | Domain → External | Use external services | Save files, send notifications |
//!
//! ### Inbound Ports ([`inbound`])
//!
//! Inbound ports define how external systems can *drive* the application.
//! They are implemented by **adapters** that translate external requests
//! into domain operations.
//!
//! ```rust,ignore
//! // A TOML adapter implements ConfigPort
//! impl ConfigPort for TomlConfigAdapter {
//!     fn load(&self, source: &str) -> Result<RawConfig> {
//!         // Parse TOML and return domain types
//!     }
//! }
//! ```
//!
//! ### Outbound Ports ([`outbound`])
//!
//! Outbound ports define how the domain can *use* external services.
//! They are implemented by **adapters** that translate domain requests
//! into external operations.
//!
//! ```rust,ignore
//! // A file adapter implements PersistPort
//! impl PersistPort for FileSystemAdapter {
//!     fn save(&self, path: &Path, theme: &Theme) -> Result<()> {
//!         // Serialize theme and write to file
//!     }
//! }
//! ```
//!
//! ## Why This Architecture?
//!
//! The hexagonal architecture provides several benefits:
//!
//! ### 1. Testability
//!
//! The domain can be tested in isolation using mock adapters:
//!
//! ```rust,ignore
//! struct MockNotifyPort;
//! impl NotifyPort for MockNotifyPort {
//!     fn send(&self, notification: Notification) -> Result<()> {
//!         // Just record the call, don't actually send
//!         Ok(())
//!     }
//! }
//!
//! #[test]
//! fn domain_sends_notification_on_error() {
//!     let notifier = MockNotifyPort;
//!     // Test domain logic with mock
//! }
//! ```
//!
//! ### 2. Flexibility
//!
//! Adapters can be swapped without changing domain logic:
//!
//! ```rust,ignore
//! // Development: Use file-based config
//! let config_port: Box<dyn ConfigPort> = Box::new(TomlConfigAdapter::new());
//!
//! // Testing: Use in-memory config
//! let config_port: Box<dyn ConfigPort> = Box::new(InMemoryConfigAdapter::new());
//! ```
//!
//! ### 3. Separation of Concerns
//!
//! Each layer has a clear responsibility:
//!
//! - **Domain**: Business rules and design system logic
//! - **Ports**: Interface contracts
//! - **Adapters**: External system integration
//!
//! ## Module Organization
//!
//! - [`inbound`] — Ports for driving the application (config loading, validation, queries)
//! - [`outbound`] — Ports for driven services (persistence, notifications, IPC)
//!
//! ## Design Principles
//!
//! Ports in fernctl follow these principles:
//!
//! 1. **Trait-based** — All ports are Rust traits, enabling static dispatch
//!    when possible and dynamic dispatch when flexibility is needed.
//!
//! 2. **Error-aware** — All fallible operations return `Result<T, FernError>`,
//!    ensuring errors are handled explicitly.
//!
//! 3. **Domain-typed** — Ports work with domain types (`Theme`, `Notification`),
//!    not external representations (raw JSON, D-Bus messages).
//!
//! 4. **Object-safe** — Ports are designed to be object-safe where possible,
//!    enabling use as trait objects (`Box<dyn ConfigPort>`).

pub mod inbound;
pub mod outbound;

// Re-export primary traits for convenience
pub use inbound::{ConfigPort, QueryPort, ValidatePort};
pub use outbound::{IpcPort, NotifyPort, PersistPort};
