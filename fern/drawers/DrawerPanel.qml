pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import Quickshell
import "backgrounds"
import "../config" as Config

// Reusable drawer panel component that encapsulates:
// - Background shape with animated corners (supports all 4 edges)
// - Animation wrapper with size-growth effect
// - Content area with proper clipping
// - HyprlandFocusGrab for click-outside-to-close
// - Escape key support
//
// Usage:
//   DrawerPanel {
//       name: "clock"
//       edge: "left"  // or "right", "top", "bottom"
//       contentWidth: 220
//
//       Column {
//           Text { text: "Hello" }
//       }
//   }
Item {
    id: root

    // Required: unique identifier matching DrawerController name
    required property string name

    // Edge determines animation direction and background shape
    property string edge: "left"

    // When true, drawer is centered on the edge (session, dashboard)
    // When false, positioned relative to trigger item (bar popouts)
    property bool centered: false

    // Computed: horizontal edges use contentWidth, vertical use contentHeight
    readonly property bool isHorizontal: edge === "left" || edge === "right"

    // Dimensions - use contentWidth for horizontal edges, contentHeight for vertical
    property real contentWidth: 200
    property real contentHeight: 200
    property real padding: Config.Theme.modulePadding

    // Content slot - children go into contentArea
    default property alias content: contentArea.data

    // Expose visibility state for content that needs it
    readonly property bool isVisible: wrapper.isVisible

    // Internal: connector radius for curved background
    readonly property real connectorRadius: Config.Theme.borderRounding * 0.75

    // Fill parent so coordinate mapping works correctly
    anchors.fill: parent

    // Background shape - rendered OUTSIDE wrapper to avoid clipping connector curves
    // Position and size depend on edge direction
    Shape {
        id: backgroundShape

        // Position offset for connectors - right edge connectors extend rightward
        x: root.edge === "left" ? wrapper.x
         : root.edge === "right" ? wrapper.x
         : wrapper.x - root.connectorRadius  // top/bottom
        y: isHorizontal ? wrapper.y - root.connectorRadius : wrapper.y

        // Size includes connector area
        // Right edge: connectors extend right, so add space on right
        // Left edge: connectors extend left, already accounted for in wrapper position
        width: root.edge === "left" ? wrapper.implicitWidth
             : root.edge === "right" ? wrapper.implicitWidth + root.connectorRadius
             : wrapper.targetWidth + root.connectorRadius * 2  // top/bottom
        height: isHorizontal
            ? wrapper.targetHeight + root.connectorRadius * 2
            : wrapper.implicitHeight

        preferredRendererType: Shape.CurveRenderer
        visible: isHorizontal ? wrapper.implicitWidth > 0 : wrapper.implicitHeight > 0
        clip: false

        // Edge-specific backgrounds - only the active edge has visible fill
        LeftBackground {
            wrapper: wrapper
            fillColor: root.edge === "left" ? Config.Theme.barBackground : "transparent"
        }
        RightBackground {
            wrapper: wrapper
            fillColor: root.edge === "right" ? Config.Theme.barBackground : "transparent"
        }
        TopBackground {
            wrapper: wrapper
            fillColor: root.edge === "top" ? Config.Theme.barBackground : "transparent"
        }
        BottomBackground {
            wrapper: wrapper
            fillColor: root.edge === "bottom" ? Config.Theme.barBackground : "transparent"
        }
    }

    // Animation wrapper - handles size-growth animation and clipping
    DrawerWrapper {
        id: wrapper

        name: root.name
        edge: root.edge
        centered: root.centered

        // Anchoring for centered drawers
        anchors.left: root.edge === "left" ? parent.left : undefined
        anchors.right: root.edge === "right" ? parent.right : undefined
        anchors.top: root.edge === "top" ? parent.top : undefined
        anchors.bottom: root.edge === "bottom" ? parent.bottom : undefined
        anchors.verticalCenter: root.centered && root.isHorizontal ? parent.verticalCenter : undefined
        anchors.horizontalCenter: root.centered && !root.isHorizontal ? parent.horizontalCenter : undefined

        // Target dimensions depend on edge
        targetWidth: isHorizontal
            ? root.connectorRadius + root.contentWidth
            : root.contentWidth
        targetHeight: isHorizontal
            ? contentArea.implicitHeight + root.padding * 2
            : root.connectorRadius + contentArea.implicitHeight + root.padding * 2

        // Content container - positioned after connector area
        Item {
            id: contentArea

            // Position depends on edge (offset by connector on the edge side)
            x: root.edge === "left" ? root.connectorRadius
             : root.edge === "right" ? 0
             : root.padding
            y: root.edge === "top" ? root.connectorRadius
             : root.edge === "bottom" ? 0
             : root.padding

            width: root.contentWidth
            implicitHeight: childrenRect.height
        }
    }

    // Note: Escape key and click-outside are handled by scrim in Drawers.qml
}
