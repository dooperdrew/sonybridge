# AGENTS.md — Mandatory Operating Guidelines for AI Agents

> You are working on **SonyBridge** (https://github.com/DooperDrew/sonybridge).
> These rules are binding for every agent and every conversation in this repository.

## 1. Mission
SonyBridge is a lightweight, responsive Windows 10/11 desktop companion and system tray application for Sony Bluetooth headphones (specifically the WH-1000XM3, WH-1000XM4, WH-1000XM5, and WF series). It solves Windows Bluetooth connection headaches and brings the full functionality of the mobile "Sony | Headphones Connect" app to the PC desktop: real-time battery monitoring, Active Noise Canceling (ANC) / Ambient Sound control, 5-band hardware Equalizer & Clear Bass tuning, quick 1-click connect/reconnect, and smart Windows audio endpoint routing.

## 2. Mandatory reading order (before any implementation)
1. `docs/01-vision.md`: problem, principles, non-goals
2. `docs/02-requirements.md`: requirement IDs (F…, NFR-…) and budgets
3. `docs/03-architecture.md`: components, boundaries, key designs
4. `docs/04-ux.md`: UI/UX spec, system tray flows, and control schema
5. `docs/05-roadmap.md`: **the single source of truth for tasks**
6. `docs/06-qa-and-validation.md`: verification matrices, NFR measurements, and empirical testing
7. `docs/m0-spike-findings.md`: verified ground truth from M0 spikes. Don't contradict or guess beyond it.
8. `docs/decisions/`: ADRs. Changing a decision requires a new ADR.
9. `docs/multi-agent-methodology.md`: parallel worker orchestration, Copilot cloud integration, and reporting templates
10. `docs/lessons-learned.md` and `CHANGELOG.md`: history and pitfalls to avoid

## 3. Continuous Learning & Self-Correction Rule
Whenever the user corrects, redirects, or clarifies a requirement:
- **Do not silently apply the change.**
- You MUST explicitly formulate the takeaway as a rule and propose writing it into `docs/lessons-learned.md` (or `AGENTS.md` and `.github/copilot-instructions.md` if binding).
- This ensures that lessons learned in one chat session immediately persist for all subsequent sessions.

## 4. Roles (multi-agent)
- **Lead agent (sole committer):** Antigravity interactive session. The **only** entity allowed to `git commit`, `git push`, tag, release, merge PRs, or close issues. Plans parallel work, creates bounded tasks in GitHub Issues, reviews worker/Copilot output, runs the verification gate, and syncs docs.
- **Worker subagents (`project-worker`):** Autonomous local subagents working in isolated branch workspaces (`Workspace: "branch"`) on **disjoint files**. **Forbidden:** `git commit`, `git push`, `git checkout main`, closing issues. They deliver results via the completion report in §8.
- **Cloud Worker agent (GitHub Copilot Agent):** Operates asynchronously in GitHub Cloud or IDE on bounded issues created via `.github/ISSUE_TEMPLATE/agent_task.yml`. Bound by `.github/copilot-instructions.md`. Delivers work exclusively via **Draft Pull Requests** on `task/<issue-id>-<description>`. **Forbidden:** pushing directly to `main`, merging PRs, or closing issues.
- Worker role names follow `<Tier>-<Domain>-<Milestone>` (e.g. `T1-Core-M2`, `T2-UI-M4`).

## 5. Complexity tiers & model selection
| Tier | Scope | Model |
|---|---|---|
| T1 Lean | pure logic, packet framing, parser, schemas, unit tests, i18n, docs, scripts | `flash_lite` / `flash` |
| T2 Moderate | Tauri UI, system tray integration, settings storage, IPC commands | `flash` / `pro` |
| T3 Critical | WinRT Bluetooth RFCOMM/SPP transport, Windows Core Audio COM routing, MDR protocol state machine | `pro` / `inherit` |

Fallback order: `inherit` → `pro` → `flash` → `flash_lite`. Every fallback **must** be logged in `docs/model-fallback-log.md`, and an audit task for it goes into the hardening milestone.

## 6. Task lifecycle & Tiered Verification Gate
- **Before:** find the task in `docs/05-roadmap.md`, restate the task + tier + target files + acceptance criteria, and check that `git status` is clean. For parallel tasks, spawn an isolated branch or worktree (`powershell -ExecutionPolicy Bypass -File .\scripts\manage-worktree.ps1 -Action New -Task <id>`). Ensure toolchain PATH is initialized (PowerShell: `$env:USERPROFILE\.cargo\bin`).
- **During:** respect the hard rules (§7). Unverified assumptions are marked 🔬 and probed, not guessed. Decouple headless tests from physical QA (keep manual QA as Pending). Workers validate their intermediate changes using the **Fast Gate**:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-gate.ps1 -Mode Fast
```
- **Draft PR / Worker Review:** Lead pulls branch or reviews worktree (`powershell -ExecutionPolicy Bypass -File .\scripts\manage-worktree.ps1 -Action Gate -Task <id> -Mode Full`), inspects diff for ownership and safety invariants, refines if needed, merges to `main` via `manage-worktree.ps1 -Action Merge` or GitHub PR, and closes the corresponding issue.
- **Gate (must pass before finishing / committing to main):**
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-gate.ps1 -Mode Full
```
- **After (lead only):** tick the roadmap box (`(YYYY-MM-DD, <commit>)`), add a CHANGELOG entry, update docs/ADRs if the design changed, add any lesson to `docs/lessons-learned.md`, commit (`M<x>.<y>: <summary>`), push, and update the corresponding GitHub Milestone and Issue. When cutting a release milestone, run `.\scripts\package-release.ps1` and `.\scripts\generate-manifests.ps1` and publish the portable `.zip` archive to GitHub Releases via `gh release create` even while physical hardware QA remains `Pending`.

## 7. Hard rules
- **Disjoint File Ownership for Parallel Tasks:** Tasks marked `⇉` MUST define explicit owned files (`crates/<crate>/**`). Workers must never edit files outside their declared ownership boundary. Cross-cutting shared files (e.g. `Cargo.toml`, workspace config) are modified exclusively by the Lead Agent.
- **Parallel Test Execution & Caching:** Automated test suites must utilize parallel workers to minimize waiting latency (`cargo test --workspace -- --test-threads=auto`). Use build toolchain caching (`sccache`).
- **GitHub Governance (Milestones, Issues & Releases):** Every roadmap milestone (`M0`..`Mn`) MUST have a corresponding GitHub Milestone and tracked GitHub Issues. Completed tasks close their issue with a commit/release reference; pending hardware QA and future enhancements stay open in their respective milestone. Every release MUST attach a downloadable portable `.zip` bundle (`SonyBridge-vX.Y.Z-windows-x64.zip`) on GitHub Releases so users can download and test without building from source.
- **Claims follow evidence:** Never claim a test, benchmark, or QA item passed without having run it. Hardware tests on physical headphones (WH-1000XM3) remain `Pending` until confirmed with empirical telemetry or user validation.
- **Keep claims in docs factual:** Avoid marketing language ("pioneering", "zero bugs") in technical specs while QA is pending.
- **Pure Core Isolation:** All Sony MDR protocol packet framing, serialization, deserialization, and state transitions live in `crates/sonybridge-core` with `#![forbid(unsafe_code)]` and zero OS or UI dependencies.
- **Resource & Memory Budget:** Idle RAM footprint must remain <= 35 MB; idle CPU must be 0% (event-driven Bluetooth I/O, no polling loops).
- **Audio Integrity First:** Never mute, detach, or switch Windows audio endpoints without graceful fallback and user consent.
- **Environment resiliency:** Never assume standard toolchains are in `$PATH`. Prepend `$env:USERPROFILE\.cargo\bin` in PowerShell commands to guard against PATH omissions in subshells.
- **Test concurrency & Global OS Objects:** Windows only permits one process to manage the tray icon or single-instance named pipe mutex. Tests touching global OS resources (pipes, sockets, registry, COM) must use unique identifiers and timeout protection to prevent test hangs.
- **Windows / PowerShell hygiene:** Files must be encoded in UTF-8 without BOM. When writing files from PowerShell, always force UTF-8 without BOM (`[System.IO.File]::WriteAllText(..., [System.Text.UTF8Encoding]::new($false))` or `Set-Content -Encoding utf8`) and set `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8` before capturing external CLI stdout (`gh`, `git`). Prefer dedicated file-writing tools over shell redirection. Use `;` rather than `&&` on Windows PowerShell 5. Avoid unconstrained recursive searches over dependency caches (`node_modules`, `target`, `.venv`). Never commit build artifacts (`target/`, `dist/`); releases go to GitHub Releases.
- **Single-Binary Distribution & Setup Ergonomics:** Release bundles must NEVER contain multiple confusing executables. Distribute a single standalone installer (`SonyBridge-Setup.exe`) and a clean portable archive (`SonyBridge-v<Version>-portable.zip`) containing `sonybridge-app.exe`, resources, and documentation. The setup wizard MUST implement a progressive flow with breadcrumbs, masking advanced options behind a toggle, and registering clean uninstallation across Windows Settings, Start Menu, and Settings GUI. Packaging tools MUST isolate every release into a dedicated, self-contained directory (`dist/v<Version>/`) containing all unpacked binaries, setup wizards, locales, documentation, and archive bundles side-by-side. The setup wizard MUST integrate a Guided Headphone Pairing Assistant directly into the completion screen.
- **Window Startup Guarantee & Silent Exit Elimination:** Desktop GUI executables must explicitly display, unminimize, and center their main window on launch (`window.show()`, `window.center()`, `window.set_focus()`); never launch unmapped in the background or exit silently when auxiliary scripts are missing.
- **Empirical Connection Truth & Dual-Bluetooth Diagnostics:** The UI must NEVER statically mock or hardcode positive connection states (`Connected`, fake battery percentage). Telemetry must default to unverified/checking on startup, and differentiate between Bluetooth Low Energy beacons (`LE_` Swift Pair) and Classic Bluetooth Audio endpoints (`WH-1000XM3`), offering proactive 1-click native pairing assistant triggers (`DevicePairingWizard.exe`, `ms-settings-connectabledevices:devicediscovery`).
- **Dynamic System Tray Telemetry:** The system tray icon and menu header MUST dynamically reflect live battery percentage, charging state, and active ANC mode (`main_tray` tooltip updates), providing instant glanceable status without opening the main window.
- **Localization:** User-facing strings go through i18n files (`crates/sonybridge-core/src/i18n.rs`, `crates/sonybridge-app/locales/{en,it}.toml`). Don't hardcode UI text.

## 8. Worker completion report (required format)
```markdown
### WORKER COMPLETION REPORT: <Role>
- Task: <ID + description>
- Model used: <model> (fallback? yes/no + reason)
- Files created/modified: <list>
- Verification: <each gate command: PASS/FAIL, test counts>
- Decisions & rationale: <numbered>
- Open issues / risks / follow-ups: <list>
```

## 9. Escalation
If reality contradicts the docs (tests, Bluetooth protocol behavior, Windows WinRT RFCOMM bugs): **stop**, report the evidence to the user, propose a probe, and update the docs only after the user agrees.

## 10. Build & run
```powershell
# Development run (Tauri GUI)
cargo tauri dev

# Run headless diagnostic probe (M0)
cargo run -p sonybridge-cli -- probe

# Workspace test suite
cargo test --workspace

# Production release build & portable ZIP packaging
cargo build --workspace --release
.\scripts\package-release.ps1 -Version "0.3.0" -SkipBuild
```
