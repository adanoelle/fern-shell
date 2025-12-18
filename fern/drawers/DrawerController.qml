pragma Singleton
import QtQuick

// Singleton that manages drawer state across the shell.
// Bar modules call showDrawer() to display their drawer panel.
QtObject {
    id: root

    // Currently active drawer name (empty string = none)
    property string currentDrawer: ""

    // Reference to the triggering bar module Item
    // Used for coordinate transformation via mapFromItem
    property Item triggerItem: null

    // Convenience property
    readonly property bool hasDrawer: currentDrawer !== ""

    // Show a drawer by name, triggered by a bar module Item
    function showDrawer(name: string, trigger: Item): void {
        if (currentDrawer === name) {
            // Toggle off if already showing
            hideDrawer();
        } else {
            // Set trigger BEFORE name - signal fires on name change
            triggerItem = trigger;
            currentDrawer = name;
            console.log("Fern: Showing drawer", name);
        }
    }

    // Hide the current drawer
    function hideDrawer(): void {
        console.log("Fern: Hiding drawer", currentDrawer);
        currentDrawer = "";
        triggerItem = null;
    }

    // Toggle a specific drawer
    function toggleDrawer(name: string, trigger: Item): void {
        if (currentDrawer === name) {
            hideDrawer();
        } else {
            showDrawer(name, trigger);
        }
    }
}
