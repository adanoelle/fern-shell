pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import "backgrounds"
import "../config" as Config

// Edge-aware drawer background loader.
// Loads the appropriate edge-specific ShapePath based on the edge property.
//
// Usage:
//   Shape {
//       DrawerBackground {
//           wrapper: myWrapper
//           edge: "left"  // or "right", "top", "bottom"
//       }
//   }
Loader {
    id: root

    // The drawer wrapper (provides animating width/height)
    required property Item wrapper

    // Edge determines which background to load
    property string edge: "left"

    // Load the appropriate edge-specific background
    sourceComponent: {
        switch (edge) {
            case "right": return rightBackground;
            case "top": return topBackground;
            case "bottom": return bottomBackground;
            default: return leftBackground;
        }
    }

    Component {
        id: leftBackground
        LeftBackground {
            wrapper: root.wrapper
        }
    }

    Component {
        id: rightBackground
        RightBackground {
            wrapper: root.wrapper
        }
    }

    Component {
        id: topBackground
        TopBackground {
            wrapper: root.wrapper
        }
    }

    Component {
        id: bottomBackground
        BottomBackground {
            wrapper: root.wrapper
        }
    }
}
