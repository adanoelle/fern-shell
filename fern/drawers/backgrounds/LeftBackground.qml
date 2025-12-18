pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import "../../config" as Config

// Curved background shape for LEFT edge drawers.
// Creates convex curved connectors at left side that
// smoothly transition from the left edge to the drawer body.
//
// Visual representation:
//
//  EDGE │    ╭───────────────╮
//       │   ╱                │
//       ├──╯                 │   <- convex curve (curves outward)
//       │                    │
//       ├──╮                 │   <- convex curve (curves outward)
//       │   ╲                │
//       │    ╰───────────────╯
//
ShapePath {
    id: root

    // The drawer wrapper (provides animating width/height)
    required property Item wrapper

    // Content dimensions from wrapper's ANIMATING implicitWidth
    readonly property real contentWidth: wrapper.implicitWidth
    readonly property real contentHeight: wrapper.implicitHeight

    // Base rounding radius
    readonly property real baseRounding: Config.Theme.borderRounding

    // Connector curve radius (the convex part connecting to edge)
    readonly property real connectorRadius: baseRounding * 0.75

    // Adaptive rounding - shrinks when width is small, grows to full radius
    readonly property bool flatten: contentWidth < baseRounding * 2
    readonly property real rounding: flatten ? Math.max(1, contentWidth / 2) : baseRounding

    // Prevent curves larger than available space
    readonly property real effectiveRounding: Math.min(rounding, contentWidth / 2, (contentHeight - connectorRadius * 2) / 2)
    readonly property real effectiveConnector: Math.min(connectorRadius, contentHeight / 4, contentWidth / 4)

    // Fill styling - fillColor should be set by parent
    strokeWidth: -1

    // Start point: at left edge, above the top convex connector
    startX: 0
    startY: -effectiveConnector

    // Top convex connector: curves outward from edge to drawer top-left
    PathArc {
        x: root.effectiveConnector
        y: 0
        radiusX: root.effectiveConnector
        radiusY: root.effectiveConnector
        direction: PathArc.Counterclockwise
    }

    // Top-left corner of drawer body
    PathArc {
        x: root.effectiveConnector + root.effectiveRounding
        y: 0
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Top edge to top-right corner
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

    // Right edge
    PathLine {
        x: root.contentWidth
        y: root.contentHeight - root.effectiveRounding
    }

    // Bottom-right corner
    PathArc {
        x: root.contentWidth - root.effectiveRounding
        y: root.contentHeight
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Bottom edge back toward left side
    PathLine {
        x: root.effectiveConnector + root.effectiveRounding
        y: root.contentHeight
    }

    // Bottom-left corner of drawer body
    PathArc {
        x: root.effectiveConnector
        y: root.contentHeight
        radiusX: root.effectiveRounding
        radiusY: root.effectiveRounding
        direction: PathArc.Clockwise
    }

    // Bottom convex connector: curves outward from drawer back to edge
    PathArc {
        x: 0
        y: root.contentHeight + root.effectiveConnector
        radiusX: root.effectiveConnector
        radiusY: root.effectiveConnector
        direction: PathArc.Counterclockwise
    }

    // Left edge back to start
    PathLine {
        x: 0
        y: -root.effectiveConnector
    }
}
