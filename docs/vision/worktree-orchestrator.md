# Git Worktree Orchestrator for Parallel Claude Experiments

## Vision

A Rust-based TUI (Terminal User Interface) that enables parallel experimentation
with Claude Code across multiple git worktrees, allowing developers to explore
different solutions simultaneously and track decision paths.

## Problem Statement

When working with Claude Code on complex problems, we often want to:

- Try multiple approaches in parallel
- Compare different solutions
- Keep failed experiments for learning
- Track why certain decisions were made
- Merge the best parts from various attempts

Currently, this requires manual worktree management and loses valuable context
about the exploration process.

## Core Concept

```
┌───────────────────────────────────────────────────┐
│           Worktree Orchestrator TUI               │
├───────────────────────────────────────────────────┤
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│ │ main    │ │ expr-1  │ │ expr-2  │ │ expr-3  │   │
│ │ ● idle  │ │ ● active│ │ ● done  │ │ ✗ failed│   │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘   │
│                                                   │
│ Current: experiment-1                             │
│ Task: "Implement workspace module with animations"│
│ Claude: Working... [████████░░] 80%               │
│                                                   │
│ Decision Log:                                     │
│ └─ Try QML animations vs CSS transitions          │
│    ├─ expr-1: QML with Behavior blocks            │
│    ├─ expr-2: CSS transitions (failed: limited)   │
│    └─ expr-3: QML with NumberAnimation            │
└───────────────────────────────────────────────────┘
```

## Features

### 1. Worktree Management

**Automated Setup**:

```rust
pub struct WorktreeManager {
    base_path: PathBuf,
    worktrees: HashMap<String, Worktree>,
}

impl WorktreeManager {
    pub fn create_experiment(&mut self, name: &str, base: &str) -> Result<Worktree> {
        // Create new worktree
        let worktree = git::create_worktree(name, base)?;

        // Initialize Claude session
        let session = ClaudeSession::new(&worktree.path);

        // Track in orchestrator
        self.worktrees.insert(name.to_string(), worktree);

        Ok(worktree)
    }
}
```

**Features**:

- One-command experiment creation
- Automatic branch naming (expr-1, expr-2, etc.)
- Clone Claude context to new experiments
- Clean up failed experiments

### 2. Parallel Claude Sessions

**Session Management**:

```rust
pub struct ClaudeSession {
    id: Uuid,
    worktree_path: PathBuf,
    status: SessionStatus,
    current_task: Option<String>,
    history: Vec<Interaction>,
}

pub enum SessionStatus {
    Idle,
    Thinking,
    Working { progress: f32 },
    Completed { success: bool },
    Failed { error: String },
}
```

**Capabilities**:

- Run multiple Claude Code instances
- Route commands to specific worktrees
- Monitor progress across sessions
- Queue tasks for sequential execution

### 3. Decision Tracking

**Decision Tree Structure**:

```rust
pub struct DecisionNode {
    id: Uuid,
    question: String,
    experiments: Vec<Experiment>,
    chosen_path: Option<ExperimentId>,
    reasoning: String,
    timestamp: DateTime<Utc>,
}

pub struct Experiment {
    id: ExperimentId,
    worktree: String,
    approach: String,
    outcome: Outcome,
    metrics: Metrics,
    claude_reasoning: String,
}

pub struct Outcome {
    status: Status,
    learnings: Vec<String>,
    artifacts: Vec<PathBuf>,
}
```

**Tracking Features**:

- Record why each experiment was started
- Capture Claude's reasoning for approaches
- Log outcomes and learnings
- Generate decision report

### 4. TUI Interface

**Layout**:

```
┌─────────────────────────┬────────────────────────┐
│     Worktree List       │    Experiment Detail   │
├─────────────────────────┼────────────────────────┤
│ [x] main                │ Name: expr-1           │
│ [>] expr-1 (active)     │ Task: Add animations   │
│ [✓] expr-2 (complete)   │ Status: In Progress    │
│ [✗] expr-3 (failed)     │                        │
│ [ ] expr-4 (queued)     │ Files Changed: 3       │
├─────────────────────────┤ - fern/modules/...     │
│     Claude Output       │ - config/...           │
├─────────────────────────┼────────────────────────┤
│ Implementing QML        │    Decision Tree       │
│ animations for smooth   ├────────────────────────┤
│ transitions...          │ Q: Animation approach? │
│                         │ ├─ QML: expr-1 ✓       │
│ [████████░░] 80%        │ ├─ CSS: expr-2 ✗       │
└─────────────────────────┴────────────────────────┘
```

**Key Bindings**:

- `n`: New experiment
- `s`: Switch worktree
- `c`: Send command to Claude
- `m`: Merge experiment to main
- `d`: Show decision tree
- `r`: Generate report

### 5. Intelligent Merging

**Merge Strategies**:

```rust
pub enum MergeStrategy {
    // Take all changes from experiment
    Full,

    // Cherry-pick specific commits
    Selective { commits: Vec<CommitId> },

    // Merge specific files only
    FileLevel { files: Vec<PathBuf> },

    // Manual review with Claude assistance
    Assisted,
}

impl Orchestrator {
    pub fn merge_experiment(&mut self, expr: &str, strategy: MergeStrategy) -> Result<()> {
        match strategy {
            MergeStrategy::Assisted => {
                // Claude helps identify best parts
                let analysis = self.analyze_experiment(expr)?;
                let suggestions = self.get_merge_suggestions(analysis)?;
                // Present to user for review
                self.present_merge_plan(suggestions)?;
            }
            // ... other strategies
        }
    }
}
```

### 6. Experiment Templates

**Predefined Exploration Patterns**:

```rust
pub struct ExperimentTemplate {
    name: String,
    description: String,
    branches: Vec<BranchTemplate>,
}

// Example templates
vec![
    ExperimentTemplate {
        name: "Performance Comparison",
        branches: vec![
            BranchTemplate { name: "baseline", approach: "Current implementation" },
            BranchTemplate { name: "optimized", approach: "With performance improvements" },
            BranchTemplate { name: "alternative", approach: "Different algorithm" },
        ],
    },
    ExperimentTemplate {
        name: "API Design",
        branches: vec![
            BranchTemplate { name: "rest", approach: "RESTful API" },
            BranchTemplate { name: "graphql", approach: "GraphQL API" },
            BranchTemplate { name: "grpc", approach: "gRPC API" },
        ],
    },
]
```

## Integration with Fern Shell

### Status Widget

Show orchestrator status in Fern:

```qml
// fern/modules/bar/components/Orchestrator.qml
Rectangle {
    property int activeExperiments: OrchestratorService.activeCount
    property int totalExperiments: OrchestratorService.totalCount

    Text {
        text: `${activeExperiments}/${totalExperiments} experiments`
        color: activeExperiments > 0 ? "#27ae60" : "#95a5a6"
    }

    MouseArea {
        onClicked: OrchestratorService.openTui()
    }
}
```

### Notification Integration

```rust
impl Orchestrator {
    fn notify_fern(&self, message: &str, urgency: Urgency) {
        // Send to Fern via MCP or D-Bus
        self.notification_service.send(Notification {
            title: "Worktree Orchestrator",
            body: message,
            urgency,
            actions: vec!["View", "Dismiss"],
        });
    }
}
```

## Workflow Examples

### Example 1: UI Component Design

```bash
# Start orchestrator
$ fern-orchestrator

# Create experiment branches
> new "Try different workspace implementations"
Created: expr-workspace-1, expr-workspace-2, expr-workspace-3

# Send different prompts to each
> claude expr-workspace-1 "Implement workspaces with smooth animations"
> claude expr-workspace-2 "Implement workspaces with instant transitions"
> claude expr-workspace-3 "Implement workspaces with spring physics"

# Compare results
> compare expr-workspace-*
┌─────────────┬───────────┬────────────┬─────────┐
│ Experiment  │ CPU Usage │ Smoothness │ LOC     │
├─────────────┼───────────┼────────────┼─────────┤
│ workspace-1 │ 2%        │ Excellent  │ 150     │
│ workspace-2 │ 1%        │ Good       │ 100     │
│ workspace-3 │ 5%        │ Excellent  │ 200     │
└─────────────┴───────────┴────────────┴─────────┘

# Merge the best approach
> merge expr-workspace-1 --strategy=full
```

### Example 2: Bug Fix Exploration

```bash
# Create debug branches
> template "Debug Performance Issue"

# Each branch tries different approach
> claude expr-debug-1 "Fix performance by optimizing QML bindings"
> claude expr-debug-2 "Fix performance by lazy loading"
> claude expr-debug-3 "Fix performance by caching"

# Combine successful fixes
> merge expr-debug-2,expr-debug-3 --strategy=selective
```

## Benefits

1. **Parallel Exploration**: Try multiple approaches simultaneously
2. **No Fear of Breaking**: Experiments are isolated
3. **Learning Preservation**: Failed attempts are documented
4. **Decision Documentation**: Understand why choices were made
5. **Optimal Solutions**: Combine best parts from experiments
6. **Time Efficiency**: Claude works on multiple solutions at once

## Technical Implementation

### Architecture

```rust
// Core orchestrator
pub struct Orchestrator {
    ui: Tui,
    worktree_manager: WorktreeManager,
    session_manager: SessionManager,
    decision_logger: DecisionLogger,
    merge_engine: MergeEngine,
    notification_service: NotificationService,
}

// TUI built with Ratatui
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app_state: AppState,
    event_handler: EventHandler,
}

// Session management
pub struct SessionManager {
    sessions: HashMap<WorktreeId, ClaudeSession>,
    executor: TokioExecutor,
    message_bus: MessageBus,
}
```

### Dependencies

```toml
[dependencies]
# TUI
ratatui = "0.28"
crossterm = "0.28"

# Git operations
git2 = "0.18"

# Async runtime
tokio = { version = "1", features = ["full"] }

# IPC with Claude
tonic = "0.11"  # gRPC for Claude communication

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI
clap = { version = "4", features = ["derive"] }
```

## Future Enhancements

1. **AI-Powered Merge Suggestions**: Claude analyzes all experiments and
   suggests optimal combination
2. **Metric Collection**: Automatic performance profiling of each experiment
3. **Experiment Replay**: Re-run previous experiment sequences
4. **Collaborative Mode**: Multiple developers + Claude on different experiments
5. **Visual Diff Viewer**: Built-in diff visualization in TUI
6. **Experiment Chains**: Sequential experiments based on previous outcomes
7. **Cost Tracking**: Monitor API usage across experiments
8. **Export Reports**: Generate markdown reports of exploration sessions

## Success Criteria

1. **Setup Time**: < 5 seconds to create new experiment
2. **Context Switching**: < 1 second to switch between worktrees
3. **Memory Usage**: < 100MB for orchestrator daemon
4. **Experiments**: Support 10+ parallel experiments
5. **History**: Preserve 100+ experiment histories

This orchestrator would transform how we work with Claude Code, enabling
systematic exploration of solution spaces while preserving the journey, not just
the destination.
