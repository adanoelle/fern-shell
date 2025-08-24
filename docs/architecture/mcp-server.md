# MCP Server Architecture for Fern Shell

## Overview

The Model Context Protocol (MCP) server enables bi-directional communication
between Claude Code and Fern Shell, allowing AI assistance to integrate
seamlessly with the desktop environment.

## What is MCP?

MCP (Model Context Protocol) is Anthropic's protocol for providing context and
tools to AI models. For Fern Shell, we'll implement an MCP server that:

- Receives commands from Claude Code
- Sends notifications to the desktop
- Provides system context to Claude
- Enables tool execution on the desktop

## Architecture

```
┌────────────────────────┐     MCP Protocol    ┌─────────────────────┐
│    Claude Code         │◄──────────────────►│   MCP Server        │
│                        │                     │   (Rust Daemon)     │
└────────────────────────┘                     └──────────┬──────────┘
                                                          │
                                               D-Bus/IPC │
                                                          ▼
┌────────────────────────────────────────────────────────────────────┐
│                         Fern Shell (QML)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Notification │  │ Status       │  │ Command      │           │
│  │ Display      │  │ Indicator    │  │ Palette      │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
└────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. MCP Server Daemon

**Rust Implementation**:

```rust
use mcp::{Server, Request, Response, Tool};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};

pub struct FernMcpServer {
    port: u16,
    ipc_client: IpcClient,
    tools: HashMap<String, Box<dyn Tool>>,
    context_providers: Vec<Box<dyn ContextProvider>>,
}

impl FernMcpServer {
    pub async fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", self.port)).await?;

        loop {
            let (socket, _) = listener.accept().await?;
            let handler = self.create_handler();

            tokio::spawn(async move {
                handler.handle_connection(socket).await;
            });
        }
    }

    async fn handle_request(&self, request: Request) -> Response {
        match request {
            Request::Tool(tool_request) => {
                self.execute_tool(tool_request).await
            },
            Request::Context(context_request) => {
                self.provide_context(context_request).await
            },
            Request::Notification(notif) => {
                self.send_notification(notif).await
            },
        }
    }
}
```

### 2. Tool Definitions

**Available Tools for Claude**:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum FernTool {
    // Send notification to desktop
    Notify {
        title: String,
        body: String,
        urgency: Urgency,
        actions: Vec<String>,
    },

    // Update status in bar
    UpdateStatus {
        message: String,
        progress: Option<f32>,
    },

    // Query system information
    GetSystemInfo {
        info_type: SystemInfoType,
    },

    // Control desktop features
    DesktopControl {
        action: DesktopAction,
    },

    // Execute command with user consent
    ExecuteCommand {
        command: String,
        requires_confirmation: bool,
    },
}

impl Tool for FernTool {
    async fn execute(&self, context: &Context) -> Result<ToolResponse> {
        match self {
            FernTool::Notify { title, body, urgency, actions } => {
                // Send via D-Bus to Fern Shell
                let notification_id = self.send_to_desktop(
                    "org.fern.Notifications",
                    "Notify",
                    &(title, body, urgency, actions)
                ).await?;

                Ok(ToolResponse::Success {
                    data: json!({ "notification_id": notification_id })
                })
            },
            // ... other tool implementations
        }
    }
}
```

### 3. Context Providers

**System Context for Claude**:

```rust
pub trait ContextProvider: Send + Sync {
    async fn provide(&self) -> Result<ContextData>;
}

pub struct DesktopContextProvider {
    ipc_client: IpcClient,
}

impl ContextProvider for DesktopContextProvider {
    async fn provide(&self) -> Result<ContextData> {
        Ok(ContextData {
            active_window: self.get_active_window().await?,
            current_workspace: self.get_current_workspace().await?,
            running_applications: self.get_running_apps().await?,
            system_status: self.get_system_status().await?,
            user_activity: self.get_activity_state().await?,
        })
    }
}

pub struct GitContextProvider {
    workspace_path: PathBuf,
}

impl ContextProvider for GitContextProvider {
    async fn provide(&self) -> Result<ContextData> {
        Ok(ContextData {
            current_branch: self.get_branch()?,
            modified_files: self.get_modified()?,
            recent_commits: self.get_recent_commits(5)?,
        })
    }
}
```

### 4. IPC Communication Layer

**D-Bus Integration**:

```rust
use zbus::{Connection, dbus_interface};

pub struct IpcClient {
    connection: Connection,
}

impl IpcClient {
    pub async fn send_notification(&self, notif: Notification) -> Result<u32> {
        let proxy = NotificationProxy::new(&self.connection).await?;
        proxy.notify(
            "Claude Code",
            notif.id,
            notif.icon,
            notif.title,
            notif.body,
            notif.actions,
            notif.hints,
            notif.timeout,
        ).await
    }

    pub async fn update_status(&self, status: Status) -> Result<()> {
        let proxy = StatusProxy::new(&self.connection).await?;
        proxy.update(status.message, status.progress).await
    }
}

#[dbus_interface(name = "org.fern.ClaudeService")]
impl FernMcpServer {
    async fn notification_action(&self, id: u32, action: String) {
        // Handle user interaction with notification
        self.handle_action(id, action).await;
    }
}
```

### 5. Security Layer

**Permission System**:

```rust
pub struct PermissionManager {
    policies: HashMap<String, Policy>,
}

#[derive(Debug, Clone)]
pub enum Policy {
    Allow,
    Deny,
    AskUser,
    AllowWithRestrictions(Vec<Restriction>),
}

impl PermissionManager {
    pub async fn check(&self, action: &Action) -> Result<bool> {
        let policy = self.policies.get(&action.tool_name)
            .unwrap_or(&Policy::AskUser);

        match policy {
            Policy::Allow => Ok(true),
            Policy::Deny => Ok(false),
            Policy::AskUser => {
                self.prompt_user(action).await
            },
            Policy::AllowWithRestrictions(restrictions) => {
                self.check_restrictions(action, restrictions)
            },
        }
    }

    async fn prompt_user(&self, action: &Action) -> Result<bool> {
        // Show dialog in Fern Shell
        let response = self.ipc_client.request_permission(
            &format!("Claude wants to: {}", action.description())
        ).await?;

        Ok(response.allowed)
    }
}
```

## QML Integration

### Service Singleton

```qml
// fern/services/ClaudeService.qml
pragma Singleton
import QtQuick
import Quickshell.DBus

Singleton {
    id: root

    property bool connected: false
    property string currentTask: ""
    property real progress: -1
    property var notifications: ListModel {}

    DBusInterface {
        service: "org.fern.ClaudeService"
        path: "/org/fern/ClaudeService"
        interface: "org.fern.ClaudeService"

        onNotificationReceived: (notification) => {
            root.notifications.append(notification);
            NotificationPopup.show(notification);
        }

        onStatusUpdated: (message, progress) => {
            root.currentTask = message;
            root.progress = progress;
        }

        function respondToPermission(requestId: string, allowed: bool) {
            call("PermissionResponse", requestId, allowed);
        }
    }

    Component.onCompleted: {
        // Connect to MCP server on startup
        connectToServer();
    }
}
```

### Notification Display

```qml
// fern/modules/ClaudeNotification.qml
Rectangle {
    id: root

    required property var notification

    implicitWidth: 300
    implicitHeight: contentColumn.implicitHeight + 20

    color: "#2c3e50"
    radius: 8

    Column {
        id: contentColumn
        padding: 10
        spacing: 5

        Text {
            text: notification.title
            font.bold: true
            color: "white"
        }

        Text {
            text: notification.body
            color: "#ecf0f1"
            wrapMode: Text.Wrap
            width: parent.width - 20
        }

        Row {
            spacing: 10

            Repeater {
                model: notification.actions

                Button {
                    text: modelData
                    onClicked: {
                        ClaudeService.handleAction(
                            notification.id,
                            modelData
                        );
                        root.close();
                    }
                }
            }
        }
    }

    // Auto-dismiss timer
    Timer {
        interval: notification.timeout || 5000
        running: notification.timeout > 0
        onTriggered: root.close()
    }
}
```

## Configuration

### Server Configuration

```toml
# ~/.config/fern/mcp-server.toml

[server]
port = 7853
host = "127.0.0.1"
max_connections = 5

[security]
require_authentication = true
auth_token_file = "~/.config/fern/mcp-token"
allowed_tools = [
    "notify",
    "update_status",
    "get_system_info"
]

[permissions]
execute_command = "ask_user"
file_system_access = "deny"
network_access = "allow_local"

[logging]
level = "info"
file = "~/.local/share/fern/mcp-server.log"
```

### Tool Registry

```json
{
  "tools": [
    {
      "name": "fern_notify",
      "description": "Send a notification to the Fern Shell desktop",
      "parameters": {
        "title": {
          "type": "string",
          "required": true
        },
        "body": {
          "type": "string",
          "required": true
        },
        "urgency": {
          "type": "enum",
          "values": ["low", "normal", "critical"],
          "default": "normal"
        }
      }
    },
    {
      "name": "fern_status",
      "description": "Update Claude's status in the Fern bar",
      "parameters": {
        "message": {
          "type": "string",
          "required": true
        },
        "progress": {
          "type": "number",
          "min": 0,
          "max": 1,
          "required": false
        }
      }
    }
  ]
}
```

## Usage Examples

### From Claude Code

```python
# Claude can use these tools
await mcp.use_tool("fern_notify", {
    "title": "Build Complete",
    "body": "All tests passed successfully!",
    "urgency": "normal",
    "actions": ["View Output", "Dismiss"]
})

await mcp.use_tool("fern_status", {
    "message": "Refactoring user.rs...",
    "progress": 0.65
})
```

### From Fern Shell

```qml
// Query Claude from Fern
ClaudeQuery {
    id: query

    function askClaude(question: string) {
        ClaudeService.query(question, {
            context: {
                activeWindow: WindowManager.activeWindow,
                currentWorkspace: Hyprland.activeWorkspace
            }
        });
    }

    onResponseReceived: (response) => {
        // Display response in UI
        responseDisplay.text = response;
    }
}
```

## Deployment

### Systemd Service

```ini
# ~/.config/systemd/user/fern-mcp-server.service
[Unit]
Description=Fern MCP Server for Claude Integration
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/fern-mcp-server
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

### Nix Package

```nix
{ pkgs, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "fern-mcp-server";
  version = "0.1.0";

  src = ./mcp-server;

  cargoSha256 = "...";

  buildInputs = with pkgs; [
    dbus
    openssl
  ];

  postInstall = ''
    install -Dm644 mcp-server.toml $out/etc/fern/mcp-server.toml
    install -Dm644 systemd/fern-mcp-server.service $out/share/systemd/user/
  '';
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_delivery() {
        let server = FernMcpServer::new_test();
        let notif = Notification {
            title: "Test".into(),
            body: "Test notification".into(),
            urgency: Urgency::Normal,
        };

        let result = server.send_notification(notif).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_check() {
        let manager = PermissionManager::new_test();
        let action = Action::ExecuteCommand {
            command: "ls".into(),
        };

        let allowed = manager.check(&action).await.unwrap();
        assert!(!allowed); // Should deny by default in tests
    }
}
```

### Integration Tests

```bash
#!/bin/bash
# Test MCP server integration

# Start server
fern-mcp-server --test-mode &
SERVER_PID=$!

# Send test notification
echo '{"tool": "notify", "params": {"title": "Test", "body": "Integration test"}}' | \
  nc localhost 7853

# Check if notification appeared
qdbus org.fern.Shell /Notifications GetLastNotification

# Clean up
kill $SERVER_PID
```

## Future Enhancements

1. **WebSocket Support**: Real-time bidirectional communication
2. **Plugin System**: Third-party tool extensions
3. **Rate Limiting**: Prevent notification spam
4. **Encryption**: End-to-end encrypted communication
5. **Multi-Model Support**: Work with different AI providers
6. **Batch Operations**: Queue and batch multiple operations
7. **Caching Layer**: Cache frequent context queries
8. **Metrics Collection**: Track usage patterns and performance

This MCP server architecture provides the foundation for deep AI integration in
Fern Shell, enabling Claude to become a true desktop assistant while maintaining
security and user control.
