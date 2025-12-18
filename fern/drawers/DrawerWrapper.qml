pragma ComponentBehavior: Bound
import QtQuick
import "../config" as Config

// Wrapper that positions and animates a single drawer panel.
// Shows when DrawerController.currentDrawer matches this wrapper's name.
// Animates size based on edge: width for left/right, height for top/bottom.
//
// The wrapper animates its size from 0 to target, creating
// an "unfold" effect. The background shape uses wrapper dimensions
// (the animating values), so corners grow organically.
Item {
    id: root

    // Unique identifier for this drawer
    required property string name

    // Edge determines animation direction and anchoring
    // "left": grows right from left edge (bar popouts)
    // "right": grows left from right edge (session, notifications)
    // "top": grows down from top edge (dashboard)
    // "bottom": grows up from bottom edge (launcher)
    property string edge: "left"

    // When true, drawer is centered on the edge (session, dashboard)
    // When false, positioned relative to trigger item (bar popouts)
    property bool centered: false

    // Computed: horizontal edges animate width, vertical animate height
    readonly property bool isHorizontal: edge === "left" || edge === "right"

    // Target dimensions when fully open (set by drawer instance)
    property real targetWidth: 0
    property real targetHeight: 0

    // Content to display in the drawer
    default property alias content: contentContainer.children

    // Connector radius (must match DrawerBackground)
    readonly property real connectorRadius: Config.Theme.borderRounding * 0.75

    // Target position - calculated when drawer becomes visible
    // For left/right edges: Y position relative to trigger
    // For top/bottom edges: X position relative to trigger
    property real targetPos: 0

    // Visibility state
    readonly property bool isVisible: DrawerController.currentDrawer === name

    // Update target position when drawer is shown (only for trigger-positioned drawers)
    Connections {
        target: DrawerController
        enabled: !root.centered
        function onCurrentDrawerChanged() {
            if (DrawerController.currentDrawer === root.name) {
                const trigger = DrawerController.triggerItem;
                if (trigger && root.parent) {
                    // Map trigger position to parent coordinates
                    const mapped = root.parent.mapFromItem(trigger, 0, 0);

                    if (root.isHorizontal) {
                        // Horizontal edges: clamp Y position
                        const minY = -root.connectorRadius;
                        const maxY = Math.max(minY, root.parent.height - root.targetHeight + root.connectorRadius);
                        root.targetPos = Math.max(minY, Math.min(mapped.y, maxY));
                    } else {
                        // Vertical edges: clamp X position
                        const minX = -root.connectorRadius;
                        const maxX = Math.max(minX, root.parent.width - root.targetWidth + root.connectorRadius);
                        root.targetPos = Math.max(minX, Math.min(mapped.x, maxX));
                    }
                }
            }
        }
    }

    // Position: centered drawers use anchoring (set externally), trigger drawers use calculated position
    // For trigger-positioned: left/right edges use y=targetPos, top/bottom use x=targetPos
    // For centered: position is 0,0 and parent handles centering via anchors
    x: centered ? 0 : (isHorizontal ? 0 : targetPos)
    y: centered ? 0 : (isHorizontal ? targetPos : 0)

    // Size animation depends on edge direction
    // Horizontal edges (left/right): animate width, fixed height
    // Vertical edges (top/bottom): animate height, fixed width
    implicitWidth: isHorizontal ? (isVisible ? targetWidth : 0) : targetWidth
    implicitHeight: isHorizontal ? targetHeight : (isVisible ? targetHeight : 0)

    // Clip reveals content progressively as size grows
    clip: true

    // Visible when animating dimension > 0
    visible: isHorizontal ? implicitWidth > 0 : implicitHeight > 0

    // Width animation for horizontal edges
    Behavior on implicitWidth {
        enabled: root.isHorizontal
        NumberAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Easing.OutCubic
        }
    }

    // Height animation for vertical edges
    Behavior on implicitHeight {
        enabled: !root.isHorizontal
        NumberAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Easing.OutCubic
        }
    }

    // Position animation
    Behavior on x {
        enabled: !root.isHorizontal
        NumberAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Easing.OutCubic
        }
    }

    Behavior on y {
        enabled: root.isHorizontal
        NumberAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Easing.OutCubic
        }
    }

    // Content container - anchoring depends on edge
    // Left: anchor left (content unfolds right)
    // Right: anchor right (content unfolds left)
    // Top: anchor top (content unfolds down)
    // Bottom: anchor bottom (content unfolds up)
    Item {
        id: contentContainer

        // Horizontal anchoring
        anchors.left: root.edge === "left" ? parent.left : undefined
        anchors.right: root.edge === "right" ? parent.right : undefined

        // Vertical anchoring
        anchors.top: root.edge === "top" || root.isHorizontal ? parent.top : undefined
        anchors.bottom: root.edge === "bottom" ? parent.bottom : undefined

        width: root.targetWidth
        height: root.targetHeight
    }
}
