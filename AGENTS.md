# AGENTS.md - S2V Collaboration Contract

**Collaboration Tier**: solo
**Adapter**: docs/s2v-adapter.md
**Master PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

This repository is initialized with S2V. Any agent entering the repo must read this file, `docs/s2v-adapter.md`, the active task spec, and the required reading listed by that task before editing code.

## Core Rules

1. Keep the task spec in `docs/specs/tasks/` as the single source of truth. Do not archive or move it.
2. Implement each task with RED -> GREEN -> optional REFACTOR -> verification -> task spec completion notes.
3. RED commits for Rust must include compileable skeletons when needed, so failing tests fail because behavior is missing, not because files or symbols are absent.
4. Before marking a task Done, run every command listed in that task's §9 Verification Plan.
5. Update task §7 traceability rows to Done and task §10 Completion Notes with real commit hashes and command evidence.
6. Update `docs/s2v-adapter.md` Task index status to Done after the task is complete.
7. If blocked after repeated attempts, write `BLOCKED-task-X.Y.md`, commit it, and continue only when another unblocked task exists.

## Commands

Use Git for Windows Bash for project-local S2V helpers when invoking shell scripts on Windows:

```bash
"C:\Program Files\Git\bin\bash.exe" -lc 'source docs/s2v/scripts/lib/preflight.sh; source docs/s2v/scripts/lib/verify.sh; s2v_verify_full "$(s2v_extract_verify_keys docs/specs/tasks/task-1.1-foundation-dataset.md)"'
```

Project verification commands are declared in `docs/s2v-adapter.md`:

- install: `cargo build`
- typecheck: `cargo check`
- unit-test: `cargo test`
- build: `cargo build`

## Solo Workflow

All commits land directly on `master` in this greenfield solo repo. Commit rhythm per task:

1. `docs(spec): task-X.Y Ready`
2. `docs(spec): task-X.Y 进入实施`
3. `test(scope): 加 task-X.Y RED 测试`
4. `feat(scope): 实现 task-X.Y`
5. optional `refactor(scope): ...`
6. `docs(spec): 回填 task-X.Y §10 Completion Notes`
7. `docs(adapter): 标记 task-X.Y 为 Done`

Never use `git reset --hard` or force push unless the user explicitly asks for history rewriting.
