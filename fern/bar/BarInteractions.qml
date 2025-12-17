import QtQuick
import "../config" as Config

// Mouse interaction layer for the bar.
// Expands bar on hover, collapses when mouse moves away.
MouseArea {
    id: root

    required property Item bar

    anchors.fill: parent
    hoverEnabled: true

    // Timer to delay collapse - gives user time to return
    Timer {
        id: collapseTimer
        interval: 250
        onTriggered: {
            if (!root.containsMouse) {
                bar.isHovered = false;
            }
        }
    }

    property real lastMouseX: 0

    onPositionChanged: {
        lastMouseX = mouseX;

        // Use fixed bar width, not animated implicitWidth, to prevent bounce during animation
        if (containsMouse && mouseX < Config.Theme.barWidth + 15) {
            // Mouse is over bar area - keep open
            collapseTimer.stop();
            bar.isHovered = true;
        }
    }

    onContainsMouseChanged: {
        if (containsMouse) {
            // Mouse entered - expand
            collapseTimer.stop();
            bar.isHovered = true;
        } else {
            // Mouse left - only collapse if NOT at the screen edge
            // If lastMouseX < 5, user is pushing against left edge - don't collapse
            if (lastMouseX > 5) {
                collapseTimer.start();
            }
        }
    }
}
