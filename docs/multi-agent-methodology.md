# Multi-Agent Parallel Architecture & Methodology Guide

> **Audience:** AI Engineers, System Architects, Open-Source Contributors, and Autonomous Agents.  
> **Status:** Canonical Methodology Guide.  
> **Repository:** `https://github.com/DooperDrew/sonybridge`

---

## 1. Executive Summary & Philosophy

When building complex software systems with autonomous AI coding agents, engineering teams typically face two major bottlenecks:
1. **Context Window Exhaustion & Cognitive Drift:** A single agent attempting to hold the entire system architecture, UI state, low-level platform APIs, and build scripts simultaneously exhausts its context window, leading to forgotten constraints, hallucinations, and regression bugs.
2. **Sequential Latency & Token Inefficiency:** Running boilerplate generation or docs updates on large, slow, high-reasoning models wastes tokens and time, while using small models on safety-critical, concurrent, or performance-sensitive paths introduces severe bugs.

This repository implements a production-grade **Hub-and-Spoke (Lead Orchestrator & Parallel Worker)** architecture. By decomposing projects into **strictly bounded, disjoint work packages** and orchestrating multi-tiered AI models across local branch worktrees and cloud agent workers, teams achieve rapid parallel development with **zero git merge collisions, deterministic verification gates, and complete traceability**.

```mermaid
flowchart TD
    USER["User / Product Owner"] -->|"High-Level Directives & Milestones"| LEAD["Lead Orchestrator Agent (Antigravity)<br/>• Sole Committer & Release Gatekeeper<br/>• Dependency Graph Analysis<br/>• Task Decomposition into GitHub Issues<br/>• Verification Gatekeeper & Docs Sync"]
    
    subgraph LOCAL_POOL["Local Worker Subagent Pool (project-worker)"]
        W1["Worker 1: Tier 1 (Flash / Flash-Lite)<br/>sonybridge-core: MDR Framing, Checksums, i18n, Unit Tests"]
        W2["Worker 2: Tier 2 (Flash / Pro)<br/>sonybridge-app: 3-Tab Tauri UI, Profiles, Setup Scripts"]
        W3["Worker 3: Tier 3 (Pro / Inherit)<br/>sonybridge-windows: WinRT RFCOMM & Core Audio IPolicyConfig"]
    end

    subgraph CLOUD_POOL["Cloud Worker Agent (GitHub Copilot Agent)"]
        COPILOT["GitHub Copilot Agent (Cloud / IDE)<br/>• Consumes agent_task.yml Issues<br/>• Generates draft code & unit tests<br/>• Delivers via Draft Pull Requests"]
    end
    
    LEAD -->|"Spawns in isolated branch"| W1
    LEAD -->|"Spawns in isolated branch"| W2
    LEAD -->|"Spawns in isolated branch"| W3
    LEAD -->|"Creates Issue via agent_task.yml"| COPILOT
    
    W1 -->|"Structured Completion Report"| LEAD
    W2 -->|"Structured Completion Report"| LEAD
    W3 -->|"Structured Completion Report"| LEAD
    COPILOT -->|"Draft PR on task/<issue-id>-<desc>"| LEAD
    
    LEAD -->|"Workspace Integration & Quality Gate"| GATE{"Lead Verification Gate<br/>1. cargo fmt --all -- --check<br/>2. cargo clippy -- -D warnings<br/>3. cargo test --workspace<br/>4. cargo build --release"}
    GATE -->|"Pass"| COMMIT["git commit & git push origin main<br/>package-release.ps1 & gh release create"]
    GATE -->|"Fail"| REJECT["Send feedback / fix locally"]
    COMMIT --> GITHUB[("GitHub main, Milestones & Releases")]
```

---

## 2. Dynamic Model Selection & Complexity Tiers

To maximize token efficiency without sacrificing code safety or accuracy, tasks are categorized into three distinct complexity tiers:

| Tier | Workload Scope | Architectural Criteria | Target Model | Token Cost Optimization |
|---|---|---|---|---|
| **Tier 1 (T1 — Lean)** | Pure MDR v1/v2 packet framing, byte-stuffing (`0x3D`), checksums, EQ/ANC opcodes, unit tests, i18n dictionaries, packaging scripts, docs. | • Pure deterministic logic (`#![forbid(unsafe_code)]`).<br>• Isolated from OS dependencies.<br>• High test coverage. | **`flash`** or **`flash_lite`** | **~80–90% reduction** in token expenditure; high velocity. |
| **Tier 2 (T2 — Moderate)** | Tauri v2 system tray lifecycle, 3-tab glassmorphic UI, sound profiles, persistent JSON config, onboarding wizard. | • UI rendering & state management.<br>• Tauri IPC commands.<br>• Standard desktop integrations. | **`flash`** or **`pro`** (task-dependent) | Balanced reasoning and throughput. |
| **Tier 3 (T3 — Complex / Critical)** | WinRT Bluetooth RFCOMM `StreamSocket` transport, Windows Core Audio `IMMDeviceEnumerator` & `IPolicyConfig` COM vtable, MDR state machine. | • Scoped COM pointer lifetimes before `CoUninitialize()`.<br>• Event-driven async I/O (`0%` idle CPU).<br>• Hardware/OS boundary safety. | **`pro`** or **`inherit`** | Deepest reasoning, absolute safety. |

### Fallback & Retrospective Audit Loop
If rate limits or quota exhaustion make a higher-tier model temporarily unavailable:
1. **Deterministic Fallback Order:** `inherit` &rarr; `pro` &rarr; `flash` &rarr; `flash_lite`.
2. **Mandatory Logging:** The Lead Agent immediately logs the incident into [`docs/model-fallback-log.md`](model-fallback-log.md).
3. **Audit Task Scheduling:** A review task is automatically created in [`docs/05-roadmap.md`](05-roadmap.md) to re-verify the implementation with a flagship model once tokens are restored.

---

## 3. The Sole Committer Protocol & Workspace Isolation

In a multi-agent environment, having multiple agents write directly to the same branch or repository root inevitably causes file corruption, race conditions, and broken builds.

### Protocol Invariants:
1. **Single Committer:** Only the Lead Agent (the interactive Antigravity session with the user) is authorized to run `git commit`, `git push`, edit git remotes, publish GitHub Releases, or close GitHub issues.
2. **Local Worktree Isolation (`scripts/manage-worktree.ps1`):** Worker subagents run in isolated workspace branches (via `Workspace: "branch"` or git worktrees under `.worktrees/<task-id>`). They touch only the files assigned to their ownership boundary.
3. **Zero Direct Main Commits:** Worker subagents and cloud agents are strictly forbidden from committing directly to `main` or merging pull requests.
4. **Disjoint File Ownership:** Parallel tasks must operate on disjoint crates/files. Cross-module dependencies are resolved sequentially or behind stable traits (`RfcommTransport`).
5. **Tiered Verification Gate (`scripts/verify-gate.ps1`):**
   - **Fast Mode (`-Mode Fast`):** Fast formatting, clippy on modified targets, and targeted unit tests with parallel runners (`--test-threads`).
   - **Full Mode (`-Mode Full`):** Comprehensive gate with full clean compilation, integration tests, and release build audit. Required before merging or releasing.

### Worktree Automation Commands:
```powershell
# 1. Spawn isolated parallel worktree for a task:
powershell -ExecutionPolicy Bypass -File .\scripts\manage-worktree.ps1 -Action New -Task "M1.1"

# 2. Run fast incremental gate inside the worktree:
powershell -ExecutionPolicy Bypass -File .\scripts\manage-worktree.ps1 -Action Gate -Task "M1.1" -Mode Fast

# 3. Merge verified branch into main (Sole Committer):
powershell -ExecutionPolicy Bypass -File .\scripts\manage-worktree.ps1 -Action Merge -Task "M1.1"

# 4. Clean up worktree:
powershell -ExecutionPolicy Bypass -File .\scripts\manage-worktree.ps1 -Action Remove -Task "M1.1"
```

---

## 4. Cloud Multi-Agent Collaboration: GitHub Copilot Agent Integration

In addition to local Antigravity subagents, SonyBridge provides a first-class workflow for **GitHub Copilot Agent** in GitHub Cloud or IDE.

```mermaid
sequenceDiagram
    autonumber
    actor User as Product Owner
    participant Lead as Lead Agent (Antigravity)
    participant GH as GitHub (Issues / PRs)
    participant Copilot as GitHub Copilot Agent
    
    User->>Lead: "Implement feature X from roadmap"
    Lead->>Lead: Decompose into bounded task + complexity tier
    Lead->>GH: Create Issue using agent_task.yml (assign files & invariants)
    GH->>Copilot: Trigger Copilot Agent on Issue
    Copilot->>Copilot: Read .github/copilot-instructions.md & codebase
    Copilot->>Copilot: Implement code + unit tests in task branch
    Copilot->>GH: Open Draft PR (task/<issue-id>-<desc>)
    GH->>Lead: Draft PR ready for review
    Lead->>Lead: Pull branch locally & run Cargo Verification Gate
    alt Tests Pass & Budgets Met
        Lead->>GH: Merge PR to main & close Issue
        Lead->>Lead: Update docs/05-roadmap.md & CHANGELOG.md
    else Regressions Detected
        Lead->>Lead: Refine code locally or comment on PR
        Lead->>GH: Push verified commit & merge
    end
```

### Division of Roles:

| Dimension | **Lead Agent (Antigravity)** | **GitHub Copilot (Agent & IDE)** |
|---|---|---|
| **Primary Role** | **Architect & Quality Gatekeeper** | **Fast Draft Implementer** |
| **Responsibilities** | • Maintains architectural integrity & specifications.<br>• Decomposes milestones into bounded GitHub Issues.<br>• Audits budgets, COM safety comments, and invariants.<br>• Runs the local verification gate and acts as Sole Committer. | • Consumes the issue specification in the cloud.<br>• Generates scaffolding, boilerplate, and initial logic.<br>• Writes unit tests for new methods.<br>• Submits self-contained Draft Pull Requests. |
| **Execution Context** | Local pair programming session with the human developer. | Asynchronous cloud execution or inline IDE assistance. |

### Configuration Guardrails:
1. [`.github/copilot-instructions.md`](../.github/copilot-instructions.md): Permanent instructions for Copilot establishing the Sole Committer Protocol, modular boundaries, and performance budgets.
2. [`.github/ISSUE_TEMPLATE/agent_task.yml`](../.github/ISSUE_TEMPLATE/agent_task.yml): Structured issue template defining task objectives, complexity tiers, file ownership boundaries, and verifiable acceptance criteria.
3. **Draft PR Pattern:** Copilot delivers all changes as Draft Pull Requests, ensuring zero unreviewed code enters `main`.

---

## 5. Worker Subagent Completion Report Template

Whenever a local worker subagent or cloud agent finishes an implementation task, it must return a structured completion report:

```markdown
### WORKER COMPLETION REPORT: <Role>
- Task: <Milestone ID and Description>
- Model Used: <Model identifier> (Fallback? Yes/No + Reason)
- Files Modified / Created:
  - <path to file 1>
  - <path to file 2>
- Verification Status:
  - Code formatting (cargo fmt --check): <PASS/FAIL>
  - Linter (cargo clippy -D warnings): <PASS/FAIL>
  - Automated tests (cargo test): <PASS/FAIL, test count>
  - Release build (cargo build --release): <PASS/FAIL>
- Key Decisions & Architectural Rationale:
  1. <Decision 1>
  2. <Decision 2>
- Open Issues / Risks / Follow-ups:
  - <Items requiring lead agent review or manual hardware testing>
```

---

## 6. Continuous Learning & Self-Correction Loop

Whenever an agent receives a user correction or discovers a toolchain/runtime quirk:
1. **Never apply the fix silently.**
2. Formulate the root cause and takeaway into a durable rule.
3. Record generic lessons in [`docs/lessons-learned.md`](lessons-learned.md).
4. If the rule is universally binding across all agents in the repository, add it to [`AGENTS.md`](../AGENTS.md) and [`.github/copilot-instructions.md`](../.github/copilot-instructions.md).
