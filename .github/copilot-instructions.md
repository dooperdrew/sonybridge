# Operating Rules & Instructions for GitHub Copilot

> **Repository:** `https://github.com/DooperDrew/xmcompanion`  
> **Role of GitHub Copilot:** Worker Subagent / Fast Draft Implementer  
> **Lead Orchestrator & Sole Committer:** Antigravity (Local Lead Agent)

Whenever working on this repository — whether via Copilot Agent, Copilot in VS Code / IDE, or GitHub Pull Request review — you MUST strictly comply with the guidelines, architecture, performance budgets, and Sole Committer protocol defined below and in [`AGENTS.md`](../AGENTS.md).

---

## 1. Division of Labor & Sole Committer Protocol

This project enforces a strict **Hub-and-Spoke (Lead Orchestrator & Worker)** multi-agent model:

```
[ User / Product Owner ]  ──► [ Lead Agent (Antigravity) ]
                                      │
        ┌─────────────────────────────┴─────────────────────────────┐
        ▼                                                           ▼
 Project Docs & Issue Prep                                  Verification & Quality Gate
(Tasks, Complexity Tiers, Owned Files)                      (Runs tests, audits budgets, merges)
        │                                                           ▲
        ▼                                                           │
 [ GitHub Issue #N (agent_task) ]                                   │
        │                                                           │
        ▼                                                           │
[ GitHub Copilot Agent ] ──────────( Draft Pull Request )───────────┘
```

### Copilot Responsibilities & Boundaries:
- **Fast Draft Implementer:** Consume bounded, self-contained GitHub Issues created from `.github/ISSUE_TEMPLATE/agent_task.yml`, generate scaffolding, implement core Sony MDR protocol logic, create Tauri UI views, and write comprehensive unit tests.
- **Draft Pull Requests Only:** Work strictly on dedicated feature branches (`task/<issue-id>-<description>`) and submit **Draft Pull Requests**.
- **PROHIBITED:**
  - ❌ Never execute `git push origin main` or attempt to commit directly to `main`.
  - ❌ Never merge pull requests into `main`.
  - ❌ Never close GitHub Issues or Milestones.
  - Only the Lead Agent (Antigravity) is authorized to run the final release gate, merge branches to `main`, package release `.zip` bundles, update project tracking, and close issues.

---

## 2. Architecture & Module Separation Boundaries

To allow parallel development across multiple agents without merge conflicts:
1. **Disjoint Module Boundaries:**
   - Independent tasks must touch disjoint files. Check the **Target Files & Ownership** section of your assigned GitHub Issue.
   - Do not edit files outside your assigned ownership scope without explicit instruction.
2. **Pure Core Logic vs. Platform/IO:**
   - All Sony MDR v1/v2 packet framing, byte-stuffing (`0x3D`), checksums, and state machines live in `crates/sonybridge-core` with `#![forbid(unsafe_code)]` and zero OS/UI dependencies.
   - Windows WinRT RFCOMM sockets and Core Audio COM (`IMMDeviceEnumerator`, `IPolicyConfig`) live exclusively in `crates/sonybridge-windows`.
3. **Zero Hardcoded Strings:**
   - All user-facing UI text, error messages, and labels must go through `crates/sonybridge-core/src/i18n.rs` and `crates/sonybridge-app/locales/{en,it}.toml`.

---

## 3. Non-Negotiable Performance & Safety Budgets

Every contribution by Copilot MUST respect the non-functional requirements defined in [`docs/02-requirements.md`](../docs/02-requirements.md):

1. **Performance Budgets (NFR-P):**
   - Idle RAM footprint must remain `<= 35 MB`; idle CPU usage must be `0.0%` (event-driven Bluetooth RFCOMM and UI IPC, zero background polling loops).
2. **Safety & Invariants (NFR-S):**
   - `crates/sonybridge-core` forbids `unsafe` code (`#![forbid(unsafe_code)]`).
   - Any `unsafe` COM/Win32 block in `crates/sonybridge-windows` must document preconditions, scope COM pointers so `Drop::drop` runs before `CoUninitialize()`, and free allocated `PWSTR` buffers with `CoTaskMemFree`.
3. **Claims Follow Evidence Rule:**
   - Mark automated unit, integration, and invariant tests as **PASS** only when verified by the verification gate.
   - Never mark physical Sony WH-1000XM3 hardware QA as passed without empirical proof. Keep physical hardware QA rows marked **`Pending`**.

---

## 4. Verification Gate & Environment Hygiene

### Toolchain PATH Resiliency & PowerShell Hygiene:
Never assume standard toolchains (`cargo`, `rustup`) are globally present in subshell `$env:PATH`. On Windows PowerShell, always prepend `$env:USERPROFILE\.cargo\bin` to `$env:PATH`. When writing files, force UTF-8 without BOM (`[System.IO.File]::WriteAllText(..., [System.Text.UTF8Encoding]::new($false))` or `Set-Content -Encoding utf8`) and set `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8` before capturing external CLI stdout.

### Single-Binary Distribution & Setup Ergonomics:
Maintain single-binary architecture (`sonybridge-app.exe` dispatches GUI, CLI diagnostic probe, setup, or uninstaller based on invocation flags) and stage standalone installer `SonyBridge-Setup.exe` alongside clean portable packages. Never commit build artifacts to git.

### Verification Gate (Must Pass 100% Locally):
Before submitting a Draft PR or marking an implementation complete, run the project verification gate:
```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

---

## 5. Draft PR Submission Checklist

When opening your Draft Pull Request:
- [ ] PR title formatted as `task/<issue-id>: <concise description>`
- [ ] Base branch is set to `main`
- [ ] PR marked as **Draft**
- [ ] PR description links the originating issue (`Closes #N`)
- [ ] All 4 Cargo verification gate commands pass cleanly
- [ ] No files outside the assigned ownership scope were touched
