# Claude Code Development Guide for Fern Shell

## Intent

This guide helps developers leverage Claude Code effectively in the Fern Shell
project. Our CLAUDE.md documentation system is specifically designed to provide
Claude (and other AI assistants) with rich context, working examples, and
architectural knowledge to accelerate development while maintaining consistency
and quality.

The goal is to make AI-assisted development not just faster, but _better_ - with
AI that understands our patterns, prevents common mistakes, and suggests
improvements based on established best practices.

## Summary

Fern Shell uses a comprehensive CLAUDE.md documentation system that acts as both
developer documentation and AI context. These files contain:

- **Working code examples** that can be copied and modified
- **Architectural decisions** with rationale
- **Pattern libraries** for common tasks
- **Cross-references** to the Caelestia shell for inspiration
- **Anti-patterns** to avoid
- **Performance guidelines** specific to QuickShell/QML

When working with Claude Code, these files are automatically available as
context, allowing the AI to understand not just what code exists, but _why_ it
was written that way and _how_ to extend it properly.

## Key Design Decisions

### 1. **Examples Over Documentation**

We prioritize complete, working code examples over abstract documentation. Every
pattern in our CLAUDE.md files can be copied and run immediately.

**Rationale**: AI assistants work best with concrete examples they can modify.
Abstract documentation often leads to incorrect interpretations.

### 2. **Hierarchical Context**

Documentation is organized from high-level (project vision) to low-level
(component specifics), allowing AI to understand context at the appropriate
level.

```
CLAUDE.md (vision/roadmap) → fern/CLAUDE.md (QML patterns) → fern/modules/CLAUDE.md (specific patterns)
```

**Rationale**: Different tasks require different context levels. Creating a new
module needs different information than planning a major feature.

### 3. **Living Templates**

Our CLAUDE.md files contain templates that evolve with the codebase. When we
discover better patterns, we update the templates.

**Rationale**: Outdated documentation is worse than no documentation. By keeping
templates current, we ensure AI suggestions remain relevant.

### 4. **Caelestia as Reference Implementation**

We explicitly reference Caelestia (a mature QuickShell configuration) throughout
our documentation as a learning resource.

**Rationale**: Caelestia has solved many problems we'll encounter. Rather than
rediscovering solutions, we can learn from their implementation.

### 5. **Error-First Troubleshooting**

TROUBLESHOOTING.md starts with exact error messages, making it easy for AI to
recognize and solve problems.

**Rationale**: When users report errors, AI can immediately match error patterns
and suggest solutions.

## Key Things to Consider

### When Working with Claude Code

1. **Start with the Right Context**

   - For new modules: "Let's create a new module. Check fern/modules/CLAUDE.md
     for the template"
   - For debugging: "I'm seeing this error: [paste error]. Check
     TROUBLESHOOTING.md"
   - For architecture: "Should we use pattern X or Y? Check CLAUDE.md for our
     decisions"

2. **Maintain Pattern Consistency**

   - Always reference existing patterns before creating new ones
   - If you need to deviate, document why in the relevant CLAUDE.md

3. **Update Documentation Immediately**

   - When you discover a new pattern, add it to the appropriate CLAUDE.md
   - When you hit an error, add it to TROUBLESHOOTING.md
   - When you make an architectural decision, document it in CLAUDE.md

4. **Use Cross-References**

   - Link between CLAUDE.md files for related concepts
   - Reference specific Caelestia files when borrowing patterns
   - Include file:line references for easy navigation

5. **Test Examples Regularly**
   ```bash
   # Verify examples still work
   for example in fern/CLAUDE.md; do
     qs -p $example
   done
   ```

### Common Workflows

**Creating a New Module:**

```
You: "I need a battery status module"
Claude: *checks fern/modules/CLAUDE.md for template*
        *checks fern/CLAUDE.md for similar examples*
        *references Caelestia's battery implementation*
        *creates module following established patterns*
```

**Debugging an Issue:**

```
You: "The bar isn't appearing"
Claude: *checks TROUBLESHOOTING.md for "bar not appearing"*
        *finds matching pattern*
        *suggests diagnostic steps*
        *provides solution*
```

**Planning a Feature:**

```
You: "How should we implement notifications?"
Claude: *checks CLAUDE.md roadmap for notification phase*
        *checks Caelestia's notification system*
        *suggests architecture based on patterns*
```

## Potential Future Directions

### 1. **AI-Generated Documentation**

- Tool to automatically update CLAUDE.md files when patterns change
- AI reviews PRs and suggests documentation updates
- Automatic example validation in CI

### 2. **Interactive Examples**

```qml
// EXAMPLE: [runnable: true, testable: true]
// This comment would trigger automated testing
```

### 3. **Pattern Mining**

- Analyze codebase for repeated patterns not yet documented
- Suggest new templates based on actual usage
- Identify deviations from documented patterns

### 4. **Context Optimization**

- Tool to determine minimal context needed for specific tasks
- Dynamic context loading based on current work
- Performance metrics for AI interactions

### 5. **Cross-Project Learning**

```markdown
<!-- IMPORT: caelestia/patterns/audio-service -->
<!-- ADAPT: rust-to-qml -->
```

### 6. **Version-Aware Documentation**

```markdown
<!-- MIN_VERSION: quickshell@0.2.0 -->
<!-- DEPRECATED: 2024-12-01, use NewPattern instead -->
```

### 7. **AI Behavior Hints**

```markdown
<!-- AI_HINT: conservative - this is performance critical -->
<!-- AI_HINT: creative - explore different approaches -->
<!-- AI_HINT: strict - must follow this pattern exactly -->
```

### 8. **Automated Troubleshooting**

- AI monitors logs and automatically updates TROUBLESHOOTING.md
- Pattern recognition for new error types
- Solution effectiveness tracking

### 9. **Living Roadmap**

- AI updates roadmap based on completed work
- Suggests next logical steps based on dependencies
- Tracks technical debt automatically

### 10. **Test Generation**

- AI generates tests from CLAUDE.md examples
- Ensures examples remain working
- Coverage reports for documented patterns

## Getting Started

1. **Read the core documentation:**

   - `CLAUDE.md` - Project vision and roadmap
   - `fern/CLAUDE.md` - QML/QuickShell patterns
   - `fern/modules/CLAUDE.md` - Module development

2. **Try an example:**

   ```bash
   # Run the clock example from fern/CLAUDE.md
   qs -p fern/modules/Clock.qml
   ```

3. **Make a change with Claude:**

   ```
   "Hey Claude, let's modify the clock module to show seconds.
   Check fern/CLAUDE.md for the Clock example."
   ```

4. **Document your patterns:** When you create something new, add it to the
   appropriate CLAUDE.md

5. **Learn from Caelestia:** Browse `/home/ada/src/nix/caelestia/` for advanced
   patterns

## Contributing to CLAUDE.md Files

When adding to CLAUDE.md files:

1. **Show, Don't Tell** - Include working code
2. **Explain Why** - Document reasoning behind decisions
3. **Link Liberally** - Cross-reference related concepts
4. **Test First** - Ensure examples work before documenting
5. **Stay Current** - Update when patterns change

Remember: These files are not just documentation - they're the shared knowledge
that makes AI assistance effective. Treat them as first-class citizens in your
development process.

## Questions?

- Check existing CLAUDE.md files for patterns
- Look at Caelestia for inspiration
- Ask Claude Code - it has all this context!
- Document new patterns you discover

Happy coding with Claude Code! 🚀
