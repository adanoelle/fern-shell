pragma Singleton
import QtQuick
import Quickshell

// Shared tick service for consolidated timers.
// Provides signals for common time intervals to avoid multiple Timer instances.
// Usage: Connect to everySecond() or everyMinute() signals instead of creating Timers.
Singleton {
    id: root

    // Signals for common intervals
    signal everySecond()
    signal everyMinute()

    // Internal counter for minute detection
    property int _seconds: 0

    // Single timer that drives all tick signals
    Timer {
        id: tickTimer
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true

        onTriggered: {
            root._seconds++;
            root.everySecond();

            if (root._seconds % 60 === 0) {
                root.everyMinute();
            }
        }
    }

    // Cleanup
    Component.onDestruction: {
        tickTimer.stop();
    }
}
