pragma Singleton
import Quickshell
import Quickshell.Hyprland
import QtQuick

// Singleton service for Hyprland integration.
// Provides workspace state and control methods.
Singleton {
    id: root

    // === WORKSPACE STATE ===

    // All workspaces
    readonly property var workspaces: Hyprland.workspaces

    // Currently focused workspace
    readonly property HyprlandWorkspace focusedWorkspace: Hyprland.focusedWorkspace

    // Active workspace ID (convenience)
    readonly property int activeWsId: focusedWorkspace?.id ?? 1

    // Currently focused monitor
    readonly property HyprlandMonitor focusedMonitor: Hyprland.focusedMonitor

    // === HELPERS ===

    // Dispatch a Hyprland command
    function dispatch(request: string): void {
        Hyprland.dispatch(request);
    }

    // Set a Hyprland keyword (config option)
    function setKeyword(keyword: string, value: string): void {
        Hyprland.dispatch(`keyword ${keyword} ${value}`);
    }

    // Configure window gaps from Fern's theme settings
    function configureGaps(gapsIn: int, gapsOut: int): void {
        setKeyword("general:gaps_in", gapsIn.toString());
        setKeyword("general:gaps_out", gapsOut.toString());
        console.log("Fern: Configured Hyprland gaps - in:", gapsIn, "out:", gapsOut);
    }

    // Configure window rounding to match Fern's border rounding
    function configureRounding(radius: int): void {
        setKeyword("decoration:rounding", radius.toString());
        console.log("Fern: Configured Hyprland rounding:", radius);
    }

    // Get monitor for a specific screen
    function monitorFor(screen: ShellScreen): HyprlandMonitor {
        return Hyprland.monitorFor(screen);
    }

    // Get active workspace ID for a specific monitor
    function activeWsIdForMonitor(screen: ShellScreen): int {
        const monitor = monitorFor(screen);
        return monitor?.activeWorkspace?.id ?? 1;
    }

    // Check if a workspace has windows
    function workspaceHasWindows(wsId: int): bool {
        for (const ws of workspaces.values) {
            if (ws.id === wsId) {
                return (ws.lastIpcObject?.windows ?? 0) > 0;
            }
        }
        return false;
    }

    // === EVENT HANDLING ===

    // Refresh state on relevant Hyprland events
    Connections {
        target: Hyprland

        function onRawEvent(event: HyprlandEvent): void {
            const eventName = event.name;

            // Workspace events
            if (["workspace", "createworkspace", "destroyworkspace",
                 "moveworkspace", "focusedmon"].includes(eventName)) {
                Hyprland.refreshWorkspaces();
                Hyprland.refreshMonitors();
            }

            // Window events (affects workspace occupancy)
            if (["openwindow", "closewindow", "movewindow",
                 "windowtitle", "activewindow"].includes(eventName)) {
                Hyprland.refreshWorkspaces();
            }
        }
    }
}
