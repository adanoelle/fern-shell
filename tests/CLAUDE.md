# Fern Shell Testing Strategy

## Purpose

This guide outlines the testing strategy for Fern Shell, covering QML
components, services, integration testing, and performance benchmarking. A
robust testing suite ensures reliability and maintainability.

## Testing Levels

```
Unit Tests → Component Tests → Integration Tests → System Tests → Performance Tests
```

## QML Component Testing

### Basic Component Test

```qml
// tests/components/FernButtonTest.qml
import QtQuick
import QtTest
import "../../fern/components"

TestCase {
    id: testCase
    name: "FernButtonTest"

    FernButton {
        id: button
        text: "Test Button"
    }

    function test_defaultState() {
        compare(button.text, "Test Button");
        verify(!button.primary);
        compare(button.implicitHeight, 32);
    }

    function test_click() {
        let clicked = false;
        button.clicked.connect(() => { clicked = true; });

        mouseClick(button);
        verify(clicked);
    }

    function test_hoverState() {
        mouseMove(button, button.width / 2, button.height / 2);
        wait(100); // Wait for animations

        // Verify hover color change
        verify(button.color !== Appearance.palette.surface);
    }

    function test_primaryButton() {
        button.primary = true;
        compare(button.color, Appearance.palette.accent);
    }
}
```

### Service Testing

```qml
// tests/services/AudioServiceTest.qml
import QtQuick
import QtTest
import "../../fern/services"

TestCase {
    name: "AudioServiceTest"

    property var mockData: {
        "volume": 0.5,
        "muted": false,
        "device": "default"
    }

    MockAudioService {
        id: audioService
    }

    function init() {
        // Reset service state before each test
        audioService.volume = mockData.volume;
        audioService.muted = mockData.muted;
    }

    function test_volumeControl() {
        audioService.setVolume(0.75);
        compare(audioService.volume, 0.75);

        // Test bounds
        audioService.setVolume(1.5);
        compare(audioService.volume, 1.0);

        audioService.setVolume(-0.5);
        compare(audioService.volume, 0.0);
    }

    function test_muteToggle() {
        verify(!audioService.muted);

        audioService.toggleMute();
        verify(audioService.muted);

        audioService.toggleMute();
        verify(!audioService.muted);
    }

    function test_volumeSignals() {
        let signalSpy = createSignalSpy(audioService, "volumeChanged");

        audioService.setVolume(0.8);
        compare(signalSpy.count, 1);
        compare(signalSpy.signalArguments[0][0], 0.8);
    }
}
```

## Integration Testing

### Module Integration Test

```qml
// tests/integration/WorkspaceModuleTest.qml
import QtQuick
import QtTest
import "../../fern/modules"
import "../../fern/services"

TestCase {
    name: "WorkspaceModuleIntegration"

    Workspaces {
        id: workspaceModule
        config: {
            "count": 5,
            "showEmpty": true
        }
        screen: mockScreen
    }

    MockHyprlandService {
        id: HyprlandService
        workspaces: [
            { id: 1, occupied: true },
            { id: 2, occupied: false },
            { id: 3, occupied: true }
        ]
    }

    QtObject {
        id: mockScreen
        property int width: 1920
        property int height: 1080
    }

    function test_workspaceDisplay() {
        // Verify correct number of workspaces shown
        compare(workspaceModule.children.length, 5);

        // Verify occupied state
        verify(workspaceModule.children[0].occupied);
        verify(!workspaceModule.children[1].occupied);
    }

    function test_workspaceSwitch() {
        let spy = createSignalSpy(HyprlandService, "workspaceSwitched");

        // Click on workspace 3
        mouseClick(workspaceModule.children[2]);

        compare(spy.count, 1);
        compare(spy.signalArguments[0][0], 3);
    }
}
```

### Bar Integration Test

```qml
// tests/integration/BarTest.qml
TestCase {
    name: "BarIntegration"

    Bar {
        id: bar
        config: testConfig
        screen: mockScreen
    }

    property var testConfig: {
        "height": 32,
        "modules": [
            { "name": "workspaces", "position": "left" },
            { "name": "clock", "position": "center" },
            { "name": "tray", "position": "right" }
        ]
    }

    function test_moduleLoading() {
        // Verify all modules loaded
        compare(bar.modules.length, 3);

        // Verify positioning
        verify(bar.modules[0].x < bar.width / 3);
        verify(bar.modules[1].x > bar.width / 3);
        verify(bar.modules[2].x > bar.width * 2 / 3);
    }

    function test_barHeight() {
        compare(bar.height, 32);
    }

    function test_responsiveLayout() {
        mockScreen.width = 1366;
        wait(100); // Wait for layout update

        // Verify modules adjusted to new width
        verify(bar.modules[2].x < 1366);
    }
}
```

## Mock Objects

### Mock Service

```qml
// tests/mocks/MockHyprlandService.qml
pragma Singleton
import QtQuick

QtObject {
    id: root

    property var workspaces: []
    property int activeWorkspace: 1
    property var activeWindow: null

    signal workspaceSwitched(int id)
    signal windowFocused(var window)

    function switchWorkspace(id: int) {
        activeWorkspace = id;
        workspaceSwitched(id);
    }

    function focusWindow(window: var) {
        activeWindow = window;
        windowFocused(window);
    }

    // Test helper
    function reset() {
        workspaces = [];
        activeWorkspace = 1;
        activeWindow = null;
    }
}
```

### Mock Data Provider

```javascript
// tests/mocks/mockData.js
const mockConfig = {
  appearance: {
    theme: 'dark',
    accentColor: '#89b4fa'
  },
  bar: {
    height: 32,
    position: 'top'
  }
}

const mockWorkspaces = [
  { id: 1, name: '1', windows: 2 },
  { id: 2, name: '2', windows: 0 },
  { id: 3, name: '3', windows: 1 }
]

function getMockWindow() {
  return {
    id: Math.random().toString(),
    title: 'Test Window',
    class: 'test-app',
    focused: false
  }
}
```

## Performance Testing

### Frame Rate Test

```qml
// tests/performance/AnimationPerformance.qml
import QtQuick
import "../fern/components"

Item {
    id: root
    width: 1920
    height: 32

    property int frameCount: 0
    property real startTime: 0
    property real fps: 0

    Repeater {
        model: 50  // Many animated components

        FernButton {
            x: index * 40
            text: index.toString()

            SequentialAnimation on opacity {
                loops: Animation.Infinite
                NumberAnimation { to: 0.5; duration: 500 }
                NumberAnimation { to: 1.0; duration: 500 }
            }
        }
    }

    Timer {
        interval: 16  // 60 FPS target
        running: true
        repeat: true
        onTriggered: {
            frameCount++;

            if (frameCount % 60 === 0) {
                let now = Date.now();
                if (startTime > 0) {
                    fps = 60000 / (now - startTime);
                    console.log("FPS:", fps.toFixed(1));

                    // Assert minimum FPS
                    if (fps < 30) {
                        console.error("Performance issue: FPS below 30");
                    }
                }
                startTime = now;
            }
        }
    }
}
```

### Memory Leak Test

```qml
// tests/performance/MemoryLeakTest.qml
TestCase {
    name: "MemoryLeakTest"

    property var components: []

    function test_componentCreationDestruction() {
        const iterations = 100;

        for (let i = 0; i < iterations; i++) {
            // Create components
            let component = Qt.createComponent("../fern/modules/Clock.qml");
            let instance = component.createObject(testCase);
            components.push(instance);
        }

        // Measure memory (pseudo-code - actual implementation would use system tools)
        let memoryBefore = getMemoryUsage();

        // Destroy all components
        components.forEach(c => c.destroy());
        components = [];

        // Force garbage collection
        gc();
        wait(1000);

        let memoryAfter = getMemoryUsage();

        // Memory should return close to original
        verify(Math.abs(memoryAfter - memoryBefore) < 1000000); // 1MB tolerance
    }
}
```

## Test Runners

### QML Test Runner

```bash
#!/bin/bash
# tests/run-qml-tests.sh

echo "Running QML Tests..."

# Component tests
qmltestrunner -input tests/components/

# Service tests
qmltestrunner -input tests/services/

# Integration tests
qmltestrunner -input tests/integration/

# Performance tests
qmltestrunner -input tests/performance/

echo "All tests completed"
```

### Nix Test Configuration

```nix
# flake-parts/checks.nix
{
  checks = {
    qml-tests = pkgs.runCommand "qml-tests" {
      buildInputs = [ pkgs.qt6.qtdeclarative ];
    } ''
      cd ${self}
      qmltestrunner -input tests/
      touch $out
    '';

    performance-tests = pkgs.runCommand "perf-tests" {} ''
      cd ${self}
      # Run performance benchmarks
      qs tests/performance/benchmark.qml
      touch $out
    '';
  };
}
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: cachix/install-nix-action@v22

      - name: Run Tests
        run: |
          nix flake check
          nix run .#tests

      - name: Performance Tests
        run: nix run .#performance-tests

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test-results/
```

## Test Coverage

### Coverage Report Generation

```javascript
// tests/coverage.js
function calculateCoverage(sourceDir, testDir) {
  const sourceFiles = findFiles(sourceDir, '*.qml')
  const testFiles = findFiles(testDir, '*Test.qml')

  let covered = 0
  let total = 0

  sourceFiles.forEach((file) => {
    const testFile = file.replace('/fern/', '/tests/') + 'Test.qml'
    if (testFiles.includes(testFile)) {
      covered++
    }
    total++
  })

  return (covered / total) * 100
}
```

## Visual Regression Testing

### Screenshot Testing

```bash
#!/bin/bash
# tests/visual-regression.sh

BASELINE_DIR="tests/screenshots/baseline"
CURRENT_DIR="tests/screenshots/current"
DIFF_DIR="tests/screenshots/diff"

# Capture current screenshots
for component in fern/components/*.qml; do
    name=$(basename $component .qml)
    qs-screenshot $component "$CURRENT_DIR/$name.png"
done

# Compare with baseline
for current in $CURRENT_DIR/*.png; do
    name=$(basename $current)
    baseline="$BASELINE_DIR/$name"
    diff="$DIFF_DIR/$name"

    if [ -f "$baseline" ]; then
        compare -metric AE "$baseline" "$current" "$diff" 2>&1
        if [ $? -ne 0 ]; then
            echo "Visual regression in $name"
        fi
    else
        echo "New baseline: $name"
        cp "$current" "$baseline"
    fi
done
```

## Best Practices

### DO ✅

- Test both happy path and edge cases
- Use mocks for external dependencies
- Test performance-critical paths
- Include visual regression tests
- Test with different configurations
- Run tests in CI/CD

### DON'T ❌

- Don't test implementation details
- Don't rely on timing for async tests
- Don't skip error cases
- Don't ignore flaky tests
- Don't test external libraries

## Test Organization

```
tests/
├── components/        # Component unit tests
├── services/         # Service unit tests
├── integration/      # Integration tests
├── performance/      # Performance benchmarks
├── mocks/           # Mock objects
├── fixtures/        # Test data
├── screenshots/     # Visual regression
└── run-tests.sh     # Test runner
```

## Debugging Tests

```bash
# Run single test
qmltestrunner -input tests/components/FernButtonTest.qml

# Run with verbose output
QT_LOGGING_RULES="qt.qml.debug=true" qmltestrunner

# Run with debugger
gdb qmltestrunner
```

## Next Steps

1. Set up test infrastructure
2. Create mock objects
3. Write component tests
4. Add integration tests
5. Implement performance benchmarks
6. Set up CI/CD
7. Add visual regression
8. Monitor coverage

Remember: Tests are your safety net - write them before you need them!
