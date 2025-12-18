pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import "../../config" as Config

// Curved background shape for BOTTOM edge drawers.
// Creates convex curved connectors at bottom side that
// smoothly transition from the bottom edge to the drawer body.
//
// Visual representation:
//
//       ╭──────────────╮
//       │              │
//       │              │
//       │    ╭────╮    │   <- convex curves (curve outward)
//       │   ╱      ╲   │
//  ─────├──╯        ╰──├─────  EDGE
//
ShapePath {
    id: root

    // The drawer wrapper (provides animating width/height)
    required property Item wrapper

    // Content dimensions from wrapper's ANIMATING implicitHeight
    readonly property real contentWidth: wrapper.implicitWidth
    readonly property real contentHeight: wrapper.implicitHeight

    // Base rounding radius
    readonly property real baseRounding: Config.Theme.borderRounding

    // Connector curve radius (the convex part connecting to edge)
    readonly property real connectorRadius: baseRounding * 0.75

    // Adaptive rounding - shrinks when height is small, grows to full radius
    readonly property bool flatten: contentHeight < baseRounding * 2
    readonly property real rounding: flatten ? Math.max(1, contentHeight / 2) : baseRounding

    // Prevent curves larger than available space
    readonly property real effectiveRounding: Math.min(rounding, contentHeight / 2, (contentWidth - connectorRadius * 2) / 2)
    readonly property real effectiveConnector: Math.min(connectorRadius, contentWidth / 4, contentHeight / 4)

    // Fill styling - fillColor should be set by parent
    strokeWidth: -1

    // Start point: at bottom edge, left of the left convex connector
    startX: -effectiveConnector
    startY: contentHeight

    // Left convex connector: curves outward from edge to drawer bottom-left
    PathArc {
        x: 0
        y: root.contentHeight - root.effectiveConnector
        radiusX: root.effectiveConnector
        radiusY: root.effectiveConnector
        direction: PathArc.Counterclockwise
    }

    // Bottom-left corner of drawer body
    PathArc {
        x: 0
        y: root.contentHeight - root.effectiveConnector - root.effectiveRounding
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Left edge to top-left corner
    PathLine {
        x: 0
        y: root.effectiveRounding
    }

    // Top-left corner
    PathArc {
        x: root.effectiveRounding
        y: 0
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Top edge
    PathLine {
        x: root.contentWidth - root.effectiveRounding
        y: 0
    }

    // Top-right corner
    PathArc {
        x: root.contentWidth
        y: root.effectiveRounding
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Right edge back toward bottom
    PathLine {
        x: root.contentWidth
        y: root.contentHeight - root.effectiveConnector - root.effectiveRounding
    }

    // Bottom-right corner of drawer body
    PathArc {
        x: root.contentWidth
        y: root.contentHeight - root.effectiveConnector
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Right convex connector: curves outward from drawer back to edge
    PathArc {
        x: root.contentWidth + root.effectiveConnector
        y: root.contentHeight
        radiusX: root.effectiveConnector
        radiusY: root.effectiveConnector
        direction: PathArc.Counterclockwise
    }

    // Bottom edge back to start
    PathLine {
        x: -root.effectiveConnector
        y: root.contentHeight
    }
}
