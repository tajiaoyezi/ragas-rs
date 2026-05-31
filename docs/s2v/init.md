---
name: s2v-init
description: |
  一站式生成完整 S2V (Spec-to-Verification) 文档集：读 PRD → 按用户选定的 Tier（solo 或 team）交互生成所有 phase spec、所有 task spec、所有 ADR、所有 BDD feature 文件，以及 adapter、AGENTS.md、目录骨架。两种 task 拆分模式（AUTO 或 STEPWISE）。强前置 PRD：缺失时提示先跑 /s2v-prd。新项目初始化时使用。
when_to_use: |
  触发：/s2v-init、"初始化 s2v"、"s2v init"、"set up s2v in this project"。跳过场景：项目已有 docs/s2v-adapter.md（建议改用 /s2v-tier 调档）。
origin: user
---

# /s2v-init — 全套 S2V 文档一站式生成命令

> 读 PRD → 选 Tier → 一次性产出**完整可用的 S2V 文档体系**：adapter / AGENTS.md / 所有 phase spec / 所有 task spec / 所有 ADR / 所有 BDD feature / 目录骨架。
>
> 完成后用户审 task spec 把 Status 改成 Ready，再跑 `/s2v-implement <task-spec>` 开干，**不需要再跑任何"补充生成"命令**。
>
> ⚠️ **task spec 留在原地不归档** —— 这是 SDD 单一事实源核心要求。用 `/s2v-implement` 实施 task spec。

---

## ⚠️ Default Profile 路径约定

本命令在步 7-11 写入文件时使用 **starter 默认路径**：

| 产物 | 默认路径 | 对应 adapter 字段 |
|---|---|---|
| Phase spec | `docs/specs/phases/phase-N-<name>.md` | `<PHASE_SPEC_PATTERN>` |
| Task spec | `docs/specs/tasks/task-X.Y-<name>.md` | `<TASK_SPEC_PATTERN>` |
| ADR | `docs/decisions/adr-NNN-<name>.md` | `<DECISION_HOME>` |
| BDD feature | `test/features/<module>.feature` | `<ACCEPTANCE_HOME>` |
| Fixture | `test/fixtures/` | `<UNIT_TEST_AREAS>` 关联 |

**项目可在 adapter 里覆盖这些路径**（full-standard §4 允许），但**首次 init 用默认路径**，因为：

1. init 跑得早，adapter 还没填业务字段，没有可读的 pattern；
2. 默认路径与 `/s2v-add`、`/s2v-implement` 的硬编码假设一致；
3. 大多数项目接受默认路径。

**如果你的项目要用非默认路径** —— ⚠️ **当前版本只完整支持 default profile**：

- `/s2v-init` 按默认路径写文件（不读 adapter pattern）
- `/s2v-add` 不按 adapter `<*_PATTERN>` 解析（仍写 default 路径）— 见 `add.md` "项目自定义路径处理"段
- `/s2v-implement` 硬假设 task spec 在 `docs/specs/tasks/*.md`

如确需自定义路径：先跑 `/s2v-init`（接受默认）→ 手动 `git mv` 改路径 → 同步更新 adapter `<*_PATTERN>` 字段（**仅作记录，不会驱动后续命令**）→ 之后每次 `/s2v-add` / `/s2v-implement` 跑完后都需手动迁移文件到自定义位置。完整 adapter pattern resolver 是后续版本目标。**不要**先改 adapter 后跑 init —— init 仍按默认写，会和 adapter 对不上。

---

## 何时用

- ✅ 新项目已有 PRD（用 `/s2v-prd` 生成）+ 想用 S2V 规范
- ❌ 项目已有 `docs/s2v-adapter.md` → 用 `/s2v-tier` 调档 / `/s2v-add` 事后追加
- ❌ 没有 PRD → 先跑 `/s2v-prd "<一句话需求>"`；init 拒绝执行无 PRD 项目

---

## 执行流程（步 0 → 13，严格按顺序）

> **步骤速查**（线性顺读执行；按步名定位，不设行号锚点以免编辑漂移 —— 行号锚点仅 `full-standard.md` §0 因 grep 查阅需要而保留）：
> 步 0 环境校验 · 步 1 PRD 强制检查 · 步 2 解析 PRD · 步 3 Tier 选择 · 步 4 Task 拆分模式 · 步 5 渲染 adapter · **步 5a §Commands × §Constraints 自检** · 步 5.5 复制规范快照到 `docs/s2v/` · 步 6 渲染 AGENTS.md · 步 7 批量 Phase Spec · 步 8 批量 Task Spec · 步 9 批量 ADR · 步 10 批量 BDD Feature · 步 11 剩余目录骨架 · 步 12 更新 adapter 索引 · 步 13 按 R6 提交 → 完工总结
> 流程后参考段：错误处理 · 与其他命令的关系 · 实施 agent 的硬约束

### 步 0：环境校验

```bash
# === S2V skill 安装目录解析（多 agent 工具适配）===
# 本 skill 不假设安装到 Claude Code 默认路径 — 按以下优先级解析实际安装位置：
#   1. S2V_SKILL_DIR 环境变量（用户/CI 显式声明）
#   2. agent runtime 注入的提示变量（CLAUDE_SKILL_DIR / SKILL_ROOT / AGENT_SKILL_PATH）
#   3. 探测 4 个常见 agent 工具默认路径
# 详见 docs/s2v/standard.md §Installation Paths by Agent（或全局 full-standard.md 对应段）
#
# ⚠️ 本函数必须 inline 在 init.md，不能 source 自全局 skill —
#    否则陷入"需要全局 skill 路径才能找到全局 skill 路径"的 bootstrap 死循环。
_s2v_skill_dir() {
  # Layer 1: 用户/CI 显式声明（最高优先级）
  if [ -n "${S2V_SKILL_DIR:-}" ] && [ -d "$S2V_SKILL_DIR" ] && [ -f "$S2V_SKILL_DIR/full-standard.md" ]; then
    echo "$S2V_SKILL_DIR"
    return 0
  fi

  # Layer 2: agent runtime 注入的提示变量（不同 agent 工具命名约定不同）
  for hint in "${CLAUDE_SKILL_DIR:-}" "${SKILL_ROOT:-}" "${AGENT_SKILL_PATH:-}"; do
    [ -z "$hint" ] && continue
    # hint 可能直接指向 s2v skill 目录，或指向 skills 父目录
    if [ -f "$hint/full-standard.md" ]; then echo "$hint"; return 0; fi
    if [ -f "$hint/s2v/full-standard.md" ]; then echo "$hint/s2v"; return 0; fi
  done

  # Layer 3: 探测常见 agent 工具默认安装路径
  for candidate in \
    "$HOME/.claude/skills/s2v" \
    "$HOME/.codex/skills/s2v" \
    "$HOME/.cursor/skills/s2v" \
    "$HOME/.agents/skills/s2v"; do      # Cursor native path / 多 agent 通用 skills 父目录
    if [ -d "$candidate" ] && [ -f "$candidate/full-standard.md" ]; then
      echo "$candidate"
      return 0
    fi
  done

  # 全部失败 — 明确指引用户
  echo "❌ 未找到 S2V skill 安装目录。" >&2
  echo "" >&2
  echo "请通过以下任一方式声明：" >&2
  echo "  (a) 显式设置环境变量：export S2V_SKILL_DIR=<your-skill-path>" >&2
  echo "  (b) 安装到常见 agent 工具默认路径之一：" >&2
  echo "        ~/.claude/skills/s2v       (Claude Code)" >&2
  echo "        ~/.codex/skills/s2v        (Codex CLI)" >&2
  echo "        ~/.cursor/skills/s2v       (Cursor — native path；也支持 ~/.agents/skills/s2v)" >&2
  echo "  (c) Aider 用户：S2V 无官方 skill 协议 — 改用 'aider --read full-standard.md'" >&2
  echo "      或在 .aider.conf.yml 配置 read: 字段引用规范" >&2
  return 2
}

S2V_SKILL_DIR="$(_s2v_skill_dir)" || exit 2
echo "✅ S2V skill 安装路径：$S2V_SKILL_DIR"
export S2V_SKILL_DIR

# === 项目环境校验 ===

# git repo 检查
test -d .git || {
  echo "⚠️ 当前不是 git 仓库。S2V 强烈推荐 git 管理。"
  echo "🅰 跑 git init 后继续（GIT_MODE=tracked）  🅱 跳过 git 直接初始化（GIT_MODE=none）"
  # 等回复；agent 内部记录 GIT_MODE 标记，步 13 按此分支处理
}

# 已初始化检查
if [ -f docs/s2v-adapter.md ]; then
  echo "❌ 已存在 docs/s2v-adapter.md。本命令仅用于全新初始化。"
  echo "调整 tier → /s2v-tier <new-tier>"
  echo "事后追加 phase/task/adr → /s2v-add <type> <name>"
  exit 1
fi
```

### 步 1：PRD 强制检查

```bash
PRD_FILES=$(find docs/prds -maxdepth 2 -name "*.prd.md" 2>/dev/null)

if [ -z "$PRD_FILES" ]; then
  echo "❌ 未检测到 PRD（docs/prds/*.prd.md）"
  echo ""
  echo "本命令需要 PRD 作为输入来批量生成全套文档。请按以下顺序："
  echo "  1. 生成 PRD：/s2v-prd \"<一句话需求>\""
  echo "  2. 完成后再跑 /s2v-init"
  echo ""
  echo "（如确实是 spike / 实验项目不想要 PRD → 用 /s2v-add 事后逐个加产物）"
  exit 1
fi

# 检测多个 PRD → 询问选哪个
PRD_COUNT=$(echo "$PRD_FILES" | wc -l)
if [ "$PRD_COUNT" -gt 1 ]; then
  echo "检测到多个 PRD，请选一个作为 Master Spec："
  # 列出 + 让用户选
fi

PRD_PATH="<selected-prd>"
echo "✅ 使用 PRD: $PRD_PATH"
```

### 步 2：解析 PRD

> ⚠️ 以下是**伪代码描述**（agent 应该用 Read 工具读 PRD + 内部解析，不要试图执行 `parse_prd` shell 函数 — 它不存在）。

> ⚠️ **PRD 表列序号 → 解析键映射**（PRD 模板用"中英双语章节锚点"，如 `## Vision｜愿景` — 任一命中即认；
> 但表内列名是中文 — agent 按**列位置而非英文键**定位下方 PRD_DATA 变量，避免用户改表头同义词时漏读）：
>
> **§Implementation Phases 表**（6 列）：
>
> | 列号 | PRD 表头中文 | 解析为 schema | 类型 |
> |---|---|---|---|
> | 1 | `#` | `phase.number` | int |
> | 2 | `Phase 名称（kebab）` | `phase.name` | string (kebab) |
> | 3 | `描述（完成后能做什么）` | `phase.description` | string |
> | 4 | `范围（涉及模块 / 文件）` | `phase.scope` | string（`+` 分隔模块）|
> | 5 | `依赖` | `phase.depends_on` | int[] 或 `-` |
> | 6 | `可并行` | `phase.parallel` | "是" / "否" + 补说明 |
>
> **§Decisions Log 表**（6 列；含"类别"列）：
>
> | 列号 | PRD 表头中文 | 解析为 schema | 类型 |
> |---|---|---|---|
> | 1 | `ID (D1, D2...)` | `decision.id` | string `D<N>` |
> | 2 | `类别` | `decision.category` | string（8 类之一，与 full-standard §16.1 对齐）|
> | 3 | `决策（一句话）` | `decision.decision` | string |
> | 4 | `选择` | `decision.choice` | string |
> | 5 | `候选方案` | `decision.alternatives` | string |
> | 6 | `拒绝候选的理由` | `decision.rationale` | string |
>
> **§Technical Risks 表**（5 列）：列号 1-5 → `risk.id / risk.description / risk.probability / risk.impact / risk.mitigation`。
>
> **行为约定**：
>
> - 任一表头被用户重命名（中文同义改写，如"范围"→"涉及范围"）→ init agent 按**列序号**继续解析（不报错）
> - 列**顺序**被改 / 列被新增 / 列被删除 → init agent 应报错 "PRD §<表名> 表列序号被修改（期望 N 列，实际 M 列），请恢复或在对话中给出新映射"

```text
# agent 的内部步骤（不可作为 shell 执行）：
# 1. 用 Read 工具读完整 PRD
# 2. 从 PRD 文本中提取以下结构化数据并存入内部变量：

PRD_DATA = {
  project_name,             # PRD title 或文件名推断
  project_type,             # PRD §Technical Approach
  user_roles,               # PRD §Users & Context
  critical_workflows[],     # PRD §User Flow / §Core Capabilities
  phases[]:                 # PRD §Implementation Phases 表
    - { number, name, description, scope, depends_on, parallel },
  decisions[]:              # PRD §Decisions Log 表
    - { id, category, decision, choice, alternatives, rationale, kebab_title (派生：从 decision 字段 kebab 化) },
  open_questions[],         # PRD §Open Questions
  tech_risks[],             # PRD §Technical Risks
}

# 3. 用对话回复展示解析结果给用户：
#    "📊 PRD 解析结果："
#    "  Project: <project_name>"
#    "  Phases: N 个 (Phase 1-N)"
#    "  Decisions Log: K 条 (D1-DK)"
```

### 步 3：Tier 选择（按决策树）

```
请选择 Collaboration Tier（只有两档）：

🅰 solo            单人 / 快速迭代 / spike 项目
                   AGENTS.md：简化版（含 S2V 必守清单 + task SOP + BLOCKED 模板）
                   git 协作：直接 main，无 PR / worktree

🅱 team（默认）    任何"非单人"协作场景：内部小团队 / 闭源 SaaS / 公开发布 / 开源
                   AGENTS.md：完整版（worktree + PR + R7 lockfile + phase smoke gate + 通知协议 + BLOCKED 完整模板）
                   git 协作：worktree + feature branch + PR-only

回复 a / b：
```

详见 `${S2V_SKILL_DIR}/tier-decision-tree.md`（步 0 已解析；Claude Code 默认 `~/.claude/skills/s2v/`，其他 agent 见 [full-standard §22 Installation Paths by Agent](full-standard.md#22-installation-paths-by-agent)）。

### 步 4：Task 拆分模式选择

```
本项目共 N 个 phase，需要拆分为 task。请选择拆分模式：

🅰 AUTO 模式 — agent 按 PRD 自动批量拆分所有 task
   适合：PRD 写得详细 / 你信任 agent 默认决策
   速度：1 次确认

🅱 STEPWISE 模式 — 每拆完一个 task 你审一次，OK 后继续下一个
   适合：你想精细控制 task 边界 / PRD 描述抽象需要补充
   速度：N × M 次确认（每 task 一次）

仍按 PRD 拆，区别只是审批节奏。回复 a / b：
```

### 步 5：渲染 adapter

按 `${S2V_SKILL_DIR}/templates/adapter.md` 渲染（步 0 已解析为实际安装路径）：
- 项目元数据（步 2 解析 + 步 3 询问补全）
- Tier 字段填入
- **基线绿三件套**（⛔ GATE 必填 — `/s2v-implement` 步 3 基线绿在**非冷启动**时调用 `s2v_run install` / `s2v_run typecheck` / `s2v_run unit-test`，且 §9 Verification 全程依赖；留 `<...>` 占位 → verify.sh `s2v_run` 占位分支（`cmd` 形如 `<...>` → hard-fail）。冷启动（首个 task 无源码/测试）由 `s2v_baseline_green` 自动跳过三者，但占位仍须先填实——后续 task / §9 必命中）：
  - `<INSTALL_COMMANDS>`：问"本项目装依赖用什么命令？" agent 推断初稿（按 PRD §Technical Approach 技术栈）：`pnpm install` / `bun install` / `pip install -r requirements.txt` / `go mod download` / `cargo build`
    - **真实命令 OR `N/A: <原因>`**（如 `N/A: 无依赖纯标准库`）；**禁 `<...>` 占位**
  - `<TYPECHECK_COMMANDS>`：问"本项目类型检查用什么命令？" agent 推断初稿（按 PRD §Technical Approach 技术栈）：`pnpm tsc --noEmit` / `bun tsc --noEmit` / `mypy .` / `go vet ./...` / `cargo check`
    - **真实命令 OR `N/A: <原因>`**（如 `N/A: 动态语言无静态类型检查`）；**禁 `<...>` 占位**
  - `<UNIT_TEST_COMMANDS>`：问"本项目跑单元测试用什么命令？" agent 推断初稿（按 PRD §Technical Approach 项目类型 + 技术栈）：`pnpm test` / `bun test` / `pytest` / `go test ./...` / `cargo test`
    - **必须真实命令**；**不接受 `<...>` 占位 / 空 / `N/A:` 形式**（adapter.md §Commands 字段语义块 "**Unit Test 强制**：§9 Verification 不接受 N/A / 留空" + verify.sh `s2v_run`：unit-test 自动置 required + required 模式下 `N/A:`/空也 hard-fail）
- **Source/Test Areas 路径锚点**（⛔ GATE — `/s2v-implement` RED/GREEN 阶段**直接当 git pathspec 用**：`Write/Edit 在 adapter §Unit test areas 列出的路径下创建测试` / `git add` 按 list 展开为多参数 / 同款 §Source areas；**必须真实路径或路径模式 — 禁 `<...>` 占位 + 禁 `N/A`**，否则 git add 失败 / agent 自由推测路径漂移）：
  - `<SOURCE_AREAS>`：问"本项目源码放哪？（可多个，每行一个 git pathspec）" agent 推断初稿（按 PRD §Technical Approach 项目类型）：`src/`（多数项目默认）/ `lib/`（npm package 风格）/ `cmd/` + `internal/`（Go 多目录，每行一个）/ `src/`（Rust）；用户可改或确认
  - `<UNIT_TEST_AREAS>`：问"本项目单元测试放哪？（可多个，每行一个 git pathspec）" agent 推断初稿（按 PRD §Technical Approach 技术栈 + 与 SOURCE_AREAS 关联）：`test/`（默认 TS/JS）/ `tests/`（Python）/ `.`（Go — 项目根全包含，等价 `git add -A`）/ `cmd/` + `internal/`（Go 多目录，每行一个，与 SOURCE_AREAS 同款）/ `src/`（Rust — `mod tests` 内嵌）/ `__tests__/`（Jest 风格）；用户可改或确认
  - **渲染规则**：到 adapter §Source And Test Areas 时**每个区域用 markdown bullet list 格式**（每行一个 path，参考 `templates/adapter.md` §Source And Test Areas schema）；下游 `/s2v-implement` 读整个 list 展开为 `git add` 多参数 — agent **不再需要手动拆引号 / 不再用空格分隔**
  - ⚠️ **Go/Rust 项目"测试与源码同包"的命名约定**（`_test.go` / `mod tests`）归 adapter §Test File Naming 段（adapter.md 已存在），**不写到本字段** — 本字段仅吃 git pathspec
  - ⚠️ **禁 `./...`**（Go package pattern — 用于 `go test ./...` / `go vet ./...` 的 command 字段，**不是 git pathspec**；实测 `git add --dry-run "./..."` → `fatal: pathspec './...' did not match any files`；上游 v5 阶段误把 Go package pattern 当 pathspec 候选写入本字段，v6 修正）
  - 这两个字段**禁 `<...>` 占位 + 禁 `N/A`**（与下游 implement.md 步 6 RED / 步 7 GREEN 把 adapter §Source And Test Areas 列表直接展开为 `git add` pathspec 一致）
- 其他 §Commands（6 项：lint / integration / e2e / build / coverage / runtime-smoke）/ §Source Areas（2 项：integration / e2e areas）/ §Constraints（6 项）字段保留占位 — 用户在完工总结指引下事后审 adapter 补全
- Phase / Task / ADR / BDD 索引留待步 7-10 填充

写入 `docs/s2v-adapter.md`。

### 步 5a：§Commands × §Constraints 一致性自检（写入后立即做，发现冲突则停下解决）

> **为什么放在这里**：步 5 刚把 §Commands（install / typecheck / unit-test，以及占位的 lint / build / coverage / runtime-smoke）和 §Constraints 同时写入 adapter。此刻是最低成本的检查窗口 —— 发现冲突只需改一个字段；若放任到首次 `/s2v-implement`，步 3「基线绿」就会 hard-fail，而修复需要跨 adapter + 所有引用该命令的 task §9 + 可能的 BDD feature 同步改动（真实案例：`go test -race` 与 `CGO_ENABLED=0` 的冲突在 init 时若发现只需改一行命令，在 implement 时发现要修复 ~14 个文件）。

**agent 的自检动作（不可跳过）**：

1. 逐条读取刚写入 adapter 的 **§Constraints 每一项**（包括从 PRD §Technical Approach / §Constraints 派生的所有约束，即便还是占位也列出其语义）。
2. 逐条读取 **§Commands 每一个已填值**（非 `<...>` 占位、非 `N/A:` 的真实命令），逐命令问："**这条命令是否违反任何一条 §Constraints？**"
3. 以下三类冲突形态是已知高发区（举例说明，不穷举）：

   | 冲突形态 | §Commands 示例 | §Constraints 示例 |
   |---|---|---|
   | **CGO/系统库 vs 编译约束** | `go test -race`（`-race` 隐式要求 cgo） | `CGO_ENABLED=0`（禁用 cgo） |
   | **网络访问 vs 离线约束** | `pip install -r requirements.txt`（需要 PyPI） | `offline: 无外网 / CI 无出口流量` |
   | **容器 / Sandbox vs 运行时约束** | `docker-compose up -d`（需要 Docker daemon） | `no Docker in CI / sandbox 环境` |

   其他常见冲突：pinned 工具版本与命令里硬写的 flags（如 `--vet=off` 被 pinned Go 版本不支持）、OS 限定命令（`pbcopy` 在 Linux CI 不存在）。

4. **发现冲突 → 立即停下，输出冲突报告并解决后再继续**：

   ```
   ⚠️ §Commands × §Constraints 冲突检测：

   命令（§Commands.<字段名>）：<命令值>
   冲突约束（§Constraints.<字段名>）：<约束值>
   冲突原因：<一句话说明为何矛盾>

   选项：
     A. 修改命令（去掉冲突 flag / 换等价命令）→ 说明新命令
     B. 修改约束（如约束已过时 / 误填）→ 说明新约束
     C. 升级到用户：约束与命令均来自 PRD，需确认哪个优先
   ```

   **选 A / B 后**：用 Edit 工具就地修改 adapter，然后重新对改动后的字段跑一遍步 5a，确认无残余冲突，再进步 5.5。

5. **无冲突 → 简短确认后继续**：`✅ §Commands × §Constraints 自检通过（无冲突）`

### 步 5.5：复制 S2V 规范快照到 `docs/s2v/`（项目自包含 — 前移以兜底 STEPWISE 中途取消）

> **为什么前移**：STEPWISE 模式（步 8）允许用户中途 abort，此时若 `docs/s2v/scripts/` 还没复制 → 后续 `/s2v-implement` 步 0 `source docs/s2v/scripts/lib/*.sh` 失败 → 用户跑不通 implement。前移到 adapter 写完即刻复制，保证任何中途状态都能跑 implement / add。
>
> **目的**：让协作者 / CI / 外部 agent（Codex / Cursor / Aider）/ 未来的自己 即使没安装全局 skill 也能读到完整规范。
>
> **为什么不用 symlink**：symlink 在协作者机器上指向不存在的路径会断；CI 容器里没有；外部 agent 拿到 repo 切换工作机会失效。复制虽有同步开销，但保证项目自包含。

```bash
# 前置创建项目目录骨架（步 7-10 写入目标 — 非 Claude agent 的 Write 工具不一定自动建父目录）
mkdir -p docs/specs/phases docs/specs/tasks docs/decisions test/features test/fixtures
mkdir -p docs/s2v/templates-used docs/s2v/scripts/lib

# 基线绿前置：创建步 5 收集进 adapter §Source And Test Areas 的真实目录
#   <SOURCE_AREAS> / <UNIT_TEST_AREAS> = 步 5 写入 adapter 的 bullet list（每行一个 pathspec），agent 在此展开为 mkdir 多参数
#   原因：全新项目首个 /s2v-implement 步 3「基线绿」按"areas 全部路径存在 且 prune
#         依赖/构建/docs 后无任何非脚手架文件"判 greenfield，并**跳过** install/
#         typecheck/unit-test（见 verify.sh s2v_baseline_green，C9'）。若单测/源目录
#         尚不存在 → all_exist=0 → **不判冷启动** → 安全侧跑门禁 → 在 greenfield 上
#         spurious 死路、主链断在首 task。故 init 先建空目录使首 task 步 3 正确识别
#         greenfield 跳过；首 task 的 RED（步 6）再往该目录写第一个失败测试。
#   ⚠️ 不创建 .gitkeep（首 task RED 会立刻写入真实测试文件；.gitkeep 即便创建也在
#      s2v_baseline_green 脚手架 denylist 内、不影响 greenfield 判定）
mkdir -p <SOURCE_AREAS> <UNIT_TEST_AREAS>

# 1. 完整规范快照
cp "${S2V_SKILL_DIR}/full-standard.md"       docs/s2v/standard.md

# 2. 实际渲染用的模板归档（按选定 tier）
cp "${S2V_SKILL_DIR}/templates/adapter.md"           docs/s2v/templates-used/adapter.md
cp "${S2V_SKILL_DIR}/templates/agents-${TIER}.md"    docs/s2v/templates-used/agents-${TIER}.md

# 3. helper 脚本（implement.md / agents-${TIER}.md §0 都通过 `source` 引用）
cp "${S2V_SKILL_DIR}/scripts/lib/adapter.sh"    docs/s2v/scripts/lib/adapter.sh
cp "${S2V_SKILL_DIR}/scripts/lib/verify.sh"     docs/s2v/scripts/lib/verify.sh
cp "${S2V_SKILL_DIR}/scripts/lib/preflight.sh"  docs/s2v/scripts/lib/preflight.sh
cp "${S2V_SKILL_DIR}/scripts/README.md"         docs/s2v/scripts/README.md

# 4. Tier 决策树（生成的 AGENTS.md / tier.md 引用 docs/s2v/tier-decision-tree.md，必须复制以闭环引用）
cp "${S2V_SKILL_DIR}/tier-decision-tree.md"     docs/s2v/tier-decision-tree.md

# 5. 在 markdown 快照头部追加"来源声明"（避免协作者误以为是源文件而直接编辑）
#    bash 脚本不加声明（声明会被 source 当注释 OK，但加了会让 git diff / shellcheck 噪音多）
SNAPSHOT_DATE=$(date +%Y-%m-%d)
for f in docs/s2v/standard.md docs/s2v/templates-used/*.md docs/s2v/scripts/README.md docs/s2v/tier-decision-tree.md; do
  TMP=$(mktemp)
  printf "> 📌 **快照来源**：本文件由 \`/s2v-init\` 在 %s 从全局 skill \`%s\` 复制。\n>\n> **请勿直接编辑此文件** — 升级 S2V 规范请改全局 skill 后重跑 \`/s2v-init\`（或手动 \`cp\` 覆盖）。\n\n---\n\n" "$SNAPSHOT_DATE" "$S2V_SKILL_DIR" > "$TMP"
  cat "$f" >> "$TMP"
  mv "$TMP" "$f"
done

echo "✅ 已复制 S2V 规范快照到 docs/s2v/（standard.md + templates-used/ + scripts/ + tier-decision-tree.md）"
```

### 步 6：渲染 AGENTS.md

按 `${S2V_SKILL_DIR}/templates/agents-${TIER}.md` 渲染（步 0 已解析为实际安装路径）。

**写入前先备份已有 AGENTS.md**（防止开源用户在已有项目接入 S2V 时无声覆盖；与 tier.md "备份旧 AGENTS.md（时间戳命名 `AGENTS.md.backup-<TS>`）" 步骤对齐）：

```bash
# 备份旧 AGENTS.md（防止误操作 — 开源用户已有项目接入 S2V 时可能已有自己的 AGENTS.md）
[ -f AGENTS.md ] && cp AGENTS.md "AGENTS.md.backup-$(date +%Y%m%d-%H%M%S)"

# 渲染 + 写入项目根 AGENTS.md（覆盖旧文件 — 已备份）
```

如果检测到已有 AGENTS.md 备份成功，agent 应在完工总结提示用户：`⚠️ 已检测到原有 AGENTS.md，已备份到 AGENTS.md.backup-<TS>；如需合并自定义内容请手动整合。`

### 步 7：批量生成所有 Phase Spec

对 PRD §Implementation Phases 表里的每一个 phase 生成：

```bash
for phase in $phases; do
  PHASE_FILE="docs/specs/phases/phase-${phase.number}-${phase.name}.md"

  渲染（基于 s2v full-standard §8.2 模板）：
  - Status: Draft（与 task spec 共用 §10.5.1 状态机，不要写 "Not Started"）
  - §1 阶段目标 ← 从 PRD phase.description
  - §2 业务价值 ← 从 PRD §Vision / §Success Metrics 推断
  - §3 涉及模块 ← 从 PRD phase.scope 推断
  - §4 任务清单 ← 留空，步 8 填充（**表格 Spec 列必须用 `../tasks/task-X.Y-name.md` phase 相对路径**，方便 IDE 内点击跳转；不要用项目根绝对路径 `docs/specs/tasks/...`）
  - §5 依赖关系 ← 从 PRD phase.depends_on / parallel
  - §6 阶段级 AC + 端到端 smoke ← <TBD-by-user>（agent 给参考）
    ⚠️ C1：§6 是受 `s2v_preflight_phase` 门禁约束的**集成兜底**，非可选说明。init 时
       留 `<TBD-by-user>`（此刻无法知道端到端 smoke），但**该 phase 的最后一个 task
       完工/合并前必须填实**（solo SOP 步 7 / team §4 Gate 3 强制；§6 仍 <TBD>/空 → BLOCK）
  - §7 阶段级风险 ← 从 PRD §Technical Risks 关联
  - §8 Phase DoD ← 标准模板
done

echo "✅ 已生成 N 个 phase spec"
```

### 步 8：批量生成所有 Task Spec（按步 4 选定模式）

#### 模式 A: AUTO

> ⚠️ 以下为**伪代码描述**，不是可执行 shell（agent 应内部循环 + 用 Write 工具生成文件）。

```text
对每个 phase ∈ $phases:
  1. 解析 phase.scope 字段，提取模块列表（agent 推断）
     # 例：phase.scope = "parser.ts + slugger.ts" → tasks = [parser, slugger]
     # 例：phase.scope = "cli.ts + io.ts + 集成测试" → tasks = [io, cli-orchestration]

  2. 对每个 task ∈ tasks:
     2a. 渲染 task spec 文件（基于 full-standard §8.3 模板，含 §5.1 Required Reading + §5.2 Imports）：
     - Status: Draft（init 默认状态，按 §10.5.1）
       ⚠️ 合法值仅：Draft / Ready / In Progress / Done / Blocked / Waived
       不要写 "Not Started" / "TODO" / "todo" / "待开始" — 不在状态机内
     - §1 Background ← 从 phase + PRD 推断
     - §2 Goal ← 标准结构 + 占位
     - §3 Scope/Out-of-Scope ← <TBD-by-user>
     - §4 Actors ← <TBD-by-user>
     - §5.1 Required Reading ← 自动列出上游 task / ADR + **对应 BDD .feature 文件**（`test/features/${task.module}.feature` — implement.md 步 0 会按 §5.1 链路读取所有引用，含 .feature；与 standard §8.3 Required Reading schema 对齐）
     - §5.2 Imports ← <TBD-by-user>
     - §5.3 函数签名 ← <TBD-by-user>
     - §6 AC ← **模式 A：完整给值 + PRD 引用标注**（详见 standard §8.3 §6 渲染规则）
       - **不**挂 `<TBD-by-user>` 前缀；不写 `- [ ] <TBD-by-user> AC1: 内容` 这种混合形式
       - §6 顶部插一段 HTML 注释（照抄 standard §8.3 §6 模板的注释，向用户说明"模式 A 渲染规则"）
       - 每条渲染为 `- [ ] **AC<N>** (PRD §<reference>): <内容>`
         - PRD 已写明 → `(PRD §<chapter>)` 精确引用
         - PRD 未写、由 task 推导 → `(本 task 新增)`
     - §7 追踪表 ← 自动生成 SCEN/TEST 编号占位
       ⚠️ §7 行级追踪 Status 列**用的是另一套状态机**（standard.md §12.2 Traceability Status），
       与文件顶部 Spec Status（§10.5.1）**不要混用**。合法值仅：
         Not Started / Spec Ready / Scenario Ready / Test Red / In Progress / Verified / Waived / Blocked / Done
       init 阶段所有行 Status 默认填 `Not Started`。
       豁免走 standard.md §12.3 流程，§7 行 Status 改 `Waived` + 在豁免登记处补五项说明。
     - §8 Risks ← 关联 PRD §Technical Risks
     - §9 Verification Plan ← 渲染规则（**按需筛减，不照搬 standard §8.3 §9 模板默认 10 项**）：
       1. **Unit Test 强制列入**（standard §8.3.1 颗粒度判据 + verify.sh `s2v_run` unit-test 自动置 required）
       2. **Install / Typecheck：adapter 对应字段非 `<...>` 占位 + 非 `N/A:` 时强制列入**（**不是 "按需" 主观判断** — implement.md 步 8 REFACTOR 阶段**只跑 unit-test**，靠 §9 兜底跑 typecheck 等；若 §9 因 "按需" 省略 Typecheck → 重构破坏类型也不报错，漏类型 gate。adapter 这两个字段步 5 已强制收集为"真实命令 OR `N/A: <原因>`"二选一 — 非 N/A 即代表项目真有此能力，必须进 §9 形成 REFACTOR 兜底；为 `N/A:` 时省略合法）
       3. **Lint / Integration / E2E / Build / Coverage / Runtime smoke / Manual** —
          ⚠️ **仅当 task 实际需要 + adapter 对应字段非 `<...>` 占位 / 非 `N/A:` 时列入**；否则**直接省略本行**
          （照抄 standard §8.3 §9 模板会把 `<...>` 占位带入 task §9 → implement.md 步 9 `s2v_verify_full` 跑到这些字段 → verify.sh `s2v_run` 占位分支 hard-fail；
            adapter 字段为 `N/A: <原因>` 时 verify.sh `s2v_run` 非 required 下 `N/A:`/空合法跳过 — 此时即使列入也不会失败，但建议直接省略保持 §9 简洁）
       4. **设计意图差异**（规则 2 vs 规则 3 — v6 r6-Codex-2 收紧）：Install/Typecheck 是**全局基础能力**（REFACTOR 必须有兜底，不留主观判断空间）；Lint/Integration/E2E/Build/Coverage/Runtime smoke/Manual 是**模块/任务级能力**（task agent 按实际需要判断，允许"task 不需要 → 即使 adapter 有也省略"）
     - §10 Completion Notes ← 必须按 standard.md §8.3 的 6 项中文 schema 占位
       字段名严格如下（缺一不可，team merge gate / CI 按这 6 个名字 grep）：
         1. **完成日期**：YYYY-MM-DD
         2. **改动文件**：（init 时占位 `<TBD-after-impl>`）
         3. **commit 列表**：（init 时占位 `<TBD-after-impl>`）
         4. **§9 Verification 结果**：（init 时占位 `<TBD-after-impl>`）
         5. **剩余风险 / 未做项**：（init 时占位 `<TBD-after-impl>`）
         6. **下游 task 影响**：（init 时占位 `<TBD-after-impl>`）

     2b. 同步追加到对应 phase 文件 `## 4. 任务清单` 表（**路径用 `../tasks/...` phase 相对路径**）：

         | <task.id> | <module> | `../tasks/task-${task.id}-${module}.md` |

         ⚠️ phase 文件位于 `docs/specs/phases/`、task 文件位于 `docs/specs/tasks/`，
         所以 `../tasks/` 是 phase 文件的正确相对路径。**不要**写项目根绝对路径
         `docs/specs/tasks/...` — IDE 跳转受限，且与 adapter §Task 总索引的"项目根视角"
         冗余。adapter 表格保持项目根路径（adapter 在项目根天然以根为参考），phase §4 表
         必须 phase 相对。

  3. agent 用对话回复展示进度："✅ AUTO 模式完成：N 个 phase × M 个 task = 共 X 个 task spec"
     "请审最终生成结果（diff），如需调整 task 边界 → 用 /s2v-add 或手动改"
```

#### 模式 B: STEPWISE

```bash
for phase in $phases; do
  echo "▶ 进入 Phase ${phase.number}: ${phase.name}"
  echo "PRD scope: ${phase.scope}"
  echo "agent 建议拆分为：${INFERRED_TASKS}"
  echo ""
  echo "请确认（或修改）："
  echo "  - 接受 agent 建议 → y"
  echo "  - 修改 task 数量/名称 → 直接说，如 '拆 3 个：parser / slugger / types'"
  # 等用户回复

  # 拆分确定后，逐 task 生成 + 用户审
  for task in $TASKS; do
    渲染 task spec（同 AUTO 模式）
    echo ""
    echo "已生成 task-${phase}.${seq}-${name}.md，请审："
    echo "  - 接受 → y，进下一个"
    echo "  - 调整 → 说出要改的字段"
    echo "  - 跳过本 task → skip"
    # 用户接受（y）后：增量追加到 adapter Task 总索引 + Phase 状态索引（伪代码）：
    #   1. Read 工具读 docs/s2v-adapter.md，grep 表头定位 `## Task 总索引` 表
    #   2. ⚠️ 先确认本 task.id 不在表内（grep `| <task.id> |`）— 重复审 / 重跑时跳过追加，避免重复行
    #   3. Edit 工具在该表最后一行之后追加一行（列序严格对齐 adapter §Task 总索引现有 6 列表头
    #      `| Task | 模块 | Spec 文件 | Status | 依赖 / Phase 内顺序 | Worktree（仅 team）|`，不新建表）：
    #      | <task.id> | <task.module> | docs/specs/tasks/task-<task.id>-<name>.md | Draft | <Phase 内顺序或 -> | <team 档填 worktree 名，solo 档 -> |
    #   4. 同款定位 `## Phase 状态索引` 表（表头 `| # | Phase | Phase Spec | Status | Tasks | Worktree（仅 team）|`），给本 phase 行的 `Tasks`（task 计数）列 +1
    # 兜底 STEPWISE 中途取消 — 即便后续步骤未跑，/s2v-implement 也能对已审 task 直接跑（adapter 索引已含对应行）
    # 步 12 末尾会按完整 phases × tasks 列表重写该两段（幂等去重 — 已追加行被新行整段替代，不会重复）
  done
done

echo "✅ STEPWISE 模式完成：共 X 个 task spec（每个已审 + 已增量追加到 adapter 索引）"
```

### 步 9：批量生成所有 ADR（自动+追问）

#### 阶段 9.1：自动从 PRD §Decisions Log 转换

> ⚠️ 以下是**伪代码 + markdown 模板混合段**（agent 内部循环 + 用 Write 工具渲染 ADR；不是可执行 shell）。

```text
ADR_INDEX=1
for decision in $decisions:
  ADR_NUM = printf "%03d" ADR_INDEX
  ADR_FILE = "docs/decisions/adr-${ADR_NUM}-${decision.kebab_title}.md"

  渲染（基于 full-standard §16.2 模板）：
  - Status: Accepted（PRD Decisions Log 默认是 Accepted）
  - Category: ${decision.category}（从 PRD §Decisions Log 类别列取值；映射到 ADR §16.2 模板的"类别"字段，与 full-standard §16.1 8 类对齐 — 待 A3 决策后类目语义最终稳定）
  - Date: 今天
  - Decided By: <user-name>（步 3 收集）
  - Related: PRD §Decisions Log D${decision.id}
  - Context: 从 PRD §Problem Statement / §Open Questions 推断
  - Decision: ${decision.choice}
  - Rationale: ${decision.rationale}
  - Alternatives: ${decision.alternatives} + 拒绝理由
  - Consequences: <TBD-by-user>（agent 给初稿）
  - Rollback: <TBD-by-user>
  - Follow-ups: 关联 PRD §Open Questions

  ADR_INDEX=$((ADR_INDEX+1))
done

echo "✅ 已从 PRD §Decisions Log 自动生成 K 个 ADR (D1-D${K} → ADR-001-${K})"
```

#### 阶段 9.2：追问遗漏（Q2B 关键）

```
检测到 PRD §Decisions Log 已转为 K 个 ADR。

S2V 规范的 ADR 通常应覆盖以下 8 类决策（**类别字面值以 full-standard §16.1「8 类决策类别（唯一权威）」表为准** — 下方括注仅释义，审计/匹配只认表内字面值）：
  1. `架构`（分层 / 抽象边界）
  2. `依赖`（核心 npm/pip 包选型）
  3. `协议接口`（CLI exit code / API 风格）
  4. `安全`（认证 / 加密 / 输入验证）
  5. `数据持久化`（schema / 迁移）
  6. `测试工具链`（测试框架 / runner）
  7. `部署发布`（npm / binary / Docker / 运行时模式）
  8. `兼容性`（OS / runtime / 数据格式 向前后兼容）

PRD 已覆盖的（按 §Decisions Log "类别"列去重 — A2 应用后已加该列）：<sorted_unique(decisions[].category)>
PRD 未覆盖的（8 类减去已覆盖 = 还差 X 类，agent 应优先建议补这些类别的 ADR）：<8 类清单 − sorted_unique(decisions[].category)>
PRD 未明示但 agent 推断常见的（基于 §Technical Approach / §Constraints）：<agent 自动检测推断 — 仅作为"未覆盖类别"的补充建议>

请确认是否补 ADR：
  - 跳过（保持 K 个） → skip
  - 加 ADR → 说明 title，如 "加 'CLI exit code 协议'"
  - 看每个建议项 → list
```

每补一个 ADR 走与阶段 9.1 同样的渲染流程，编号续接。

### 步 10：批量生成所有 BDD Feature 文件

> ⚠️ 以下为**伪代码描述**（agent 内部循环 + 用 Read/Write 工具实现"存在则追加 / 不存在则新建"，不要试图执行 shell `if [ -f ... ]`）。

```text
# 每个 module 一个 .feature（与 adapter.md §Source And Test Areas: BDD feature 命名约定 `<module>.feature` 一致）
# 同 module 跨多 task 时 → **追加新 task 的 Scenarios 到现有文件，不覆盖**
#   （task.module 可能跨多个 phase/task 重复，如 task-1.1-parser + task-2.1-parser 共享 module=parser）

unique_modules = distinct(all_tasks[].module)

for module in $unique_modules:
  FEATURE_FILE = "test/features/${module}.feature"
  tasks_in_module = filter(all_tasks, task.module == module)

  if FEATURE_FILE 已存在（用 Read 工具检查）:
    # 追加场景模式：保留现有 Feature 头 + 已有 Scenarios，只追加新 task 的 Scenarios
    对 tasks_in_module 中尚未在文件中出现的 task.id（按 grep "Maps to: .*task-${task.id}-" 判定）:
      在文件末尾追加：
        # ---
        # Maps to: docs/specs/tasks/task-${task.id}-${module}.md

        Scenario: SCEN-${task.id}.1 — <TBD-by-user，对应 AC1>
          Given <TBD>
          When <TBD>
          Then <TBD>

        # AC2 / AC3 / ... 各预留一个 SCEN
  else:
    # 新文件模式：完整渲染 Feature 头 + 所有 tasks_in_module 的 Scenarios（轻量 BDD，s2v §9.2 模板）
    渲染：
      # language: en
      # Maps to:
      #   - docs/specs/tasks/task-${task_1.id}-${module}.md
      #   - docs/specs/tasks/task-${task_2.id}-${module}.md   （若 module 跨多 task）

      Feature: ${module}
        In order to <TBD-by-user>
        As <TBD-by-user>
        I want <module 职责 — 从 PRD §Technical Approach.关键模块边界 抽取>

      # 对每个 task ∈ tasks_in_module 渲染一组 Scenarios：
      #   Scenario: SCEN-${task.id}.1 — <TBD-by-user，对应 AC1>
      #     Given <TBD> / When <TBD> / Then <TBD>
      #   # AC2 / AC3 / ... 各预留一个 SCEN
done

echo "✅ 已生成 X 个 .feature 文件（每 module 一个；含占位 SCEN，等 task agent 实施时填入）"
```

### 步 11：创建剩余目录骨架（规范快照已在步 5.5 完成）

```bash
mkdir -p test/fixtures
touch test/fixtures/.gitkeep

# 1. 项目根无 .gitignore → 写一个 baseline（防止 IDE 配置 / 构建产物误 commit）
#    init 完成 → Phase 1 实施之间有时间窗口，如 .idea/.vscode/ 已被 IDE 创建会裸奔。
#    ⚠️ 必须语言感知：baseline 含通用 + 多生态构建产物；agent 还要按步 2 解析的
#       project_type / 技术栈在 "# 语言特定" 段**补该语言条目**（否则如 Python 项目
#       跑测试产生的 __pycache__/*.pyc 会随 implement 步 6 `git add <UNIT_TEST_AREAS>`
#       入库，污染历史 + 违背本段"防构建产物误 commit"目的）。
#    ⚠️ 补**编译型二进制**条目时必须 **repo 根锚定**（前缀 `/`，如 `/goflow`，
#       或集中到 `/bin/`）：裸名 `goflow`（无前导 /）会匹配**任意层级**同名路径 ——
#       Go 惯用 `cmd/<binary>/` 布局下会**静默吞掉源码目录** `cmd/goflow/`（git add
#       对忽略文件静默跳过、不报错）→ implement 步 6 RED 提交丢源码 / 假绿。与
#       D-3/C10 同类（忽略规则静默脱 track）；fixture 有 s2v_guard_fixture_tracked
#       兜底，**源码目录目前无机械兜底**，故此处务必锚定、勿用裸二进制名。
if [ ! -f .gitignore ]; then
  cat > .gitignore << 'EOF'
# 依赖与构建产物（通用 + 多生态）
node_modules/
dist/
build/
coverage/
target/
vendor/

# 日志与临时文件
*.log
*.tmp
.DS_Store

# S2V fixture 是测试事实源 —— 必须入库，豁免上面的通用忽略（*.log/*.tmp 等）
# （D-3：日志/数据类项目 fixture 常用 .log/.csv，被 *.log 吞 → /s2v-add 静默脱 track）
!test/fixtures/
!test/fixtures/**

# IDE / 编辑器配置
.idea/
.vscode/
*.swp

# 语言特定（init 按步 2 解析的技术栈补全 — 下列为常见默认，按需增删）
# Python:
__pycache__/
*.py[cod]
.pytest_cache/
.mypy_cache/
.venv/
venv/
*.egg-info/
# JVM:
*.class
# 其它：按 PRD §Technical Approach 技术栈补（如 Rust target/ 已在上方、Go bin/ 等）
EOF
  echo "✅ 已写入 baseline .gitignore（语言感知；如技术栈特殊请按步 2 解析结果补条目）"
fi

# 2. tier=team 时追加 S2V 协作临时文件（solo 不需要 STATUS-MAIN.md）
if [ "$TIER" != "solo" ]; then
  if ! grep -q "STATUS-MAIN.md" .gitignore 2>/dev/null; then
    cat >> .gitignore << 'EOF'

# S2V 协作临时文件
STATUS-MAIN.md
EOF
  fi
fi
```

<!-- 11b 已合并到步 5.5（规范快照前移以兜底 STEPWISE 中途取消） — 此处保留锚点提示，不再独立编号 -->

### 步 12：更新 adapter 索引（步 5 留的占位现在填入；STEPWISE 模式下已部分追加的 Task / Phase 索引会被完整重写，幂等去重）

把步 7-10 生成的全部 phase / task / ADR / feature 列表写入 adapter 的：
- `## Phase 状态索引` 表
- `## Task 总索引` 表
- `## ADR 索引` 表
- `## BDD Feature 索引` 表

> **幂等去重实现**（伪代码）：
> 1. 以步 7-10 的完整 phases × tasks × ADR × feature 列表为准（**单一事实源**），不依赖表内现有行
> 2. 按 `task.id`（或文件路径）`sorted_unique` 生成每段的完整行集合
> 3. 用 Edit 工具**整段替换**对应表 body（保留表头），而非追加 — STEPWISE 模式 accept loop 已增量追加的部分行被本步整段覆盖，重复行自然消除（幂等成立）
> 4. AUTO 模式下表内本无行，整段写入即可（同一代码路径，无需分支）

### 步 13：按 R6 流程提交

#### 13a: GIT_MODE=none（步 0 选 B「跳过 git」）

- **不跑任何 git 命令**
- 输出"未提交产物清单"提示用户：

  ```
  ⚠️ 本次未启用 git 管理（步 0 选 B）。S2V 强烈推荐 git 管理 — 如需启用，请手动跑：
    git init
    git add -A
    git commit -m "chore: 初始化 S2V 全套文档（tier=${TIER}）"

  本次生成的产物清单（未跟踪）：
    - docs/s2v-adapter.md
    - AGENTS.md
    - docs/specs/phases/   （N 个 phase spec）
    - docs/specs/tasks/    （M 个 task spec）
    - docs/decisions/      （K 个 ADR）
    - test/features/       （X 个 .feature 文件）
    - docs/s2v/            （规范快照 — standard.md + scripts/ + templates-used/）
    - .gitignore           （baseline，含 STATUS-MAIN.md 仅 team 档）
  ```

#### 13b: GIT_MODE=tracked（步 0 选 A，或已是 git repo）

- **solo 档**：`git add -A && git commit -m "chore: 初始化 S2V 全套文档（tier=solo, N phase / M task / K ADR）"`
- **team 档**：按 R6.1 PR-only 协议（规范实现 — `/s2v-tier` 步 5 / `/s2v-add` 统一动作 inline 同款；完整协议见 [`references/r6-pr-protocol.md`](references/r6-pr-protocol.md)）— 有 remote → push + `gh pr create`（开源/团队真实 PR-only）；无 remote → 本地 PR 模拟
  ```bash
  # R6.1 PR-only 协议（详见 references/r6-pr-protocol.md — init / tier / add 三命令同款）
  BRANCH=chore/s2v-init
  TITLE="chore: 初始化 S2V 全套文档（tier=${TIER}）"
  BODY="一次性生成全套 S2V 文档（adapter / AGENTS.md / phase spec / task spec / ADR / BDD feature / 规范快照）。审核 PR 后 merge 到 main。"

  BASE_BRANCH=$(git branch --show-current)   # 捕获当前 trunk（stock git init 默认 master，禁止写死 main）
  git checkout -b "$BRANCH"
  git add -A
  git commit -m "$TITLE"

  if git remote -v | grep -q "^origin"; then
    # 有 remote → push + gh pr create（开源 / 团队场景的真实 PR-only）
    git push -u origin "$BRANCH"
    echo "✅ 已 push $BRANCH 到 origin。请开 PR（命令行 OR Web）："
    echo "  gh pr create --base main --title '$TITLE' --body '$BODY'"
    echo "  （若 gh 未安装：在 GitHub/GitLab Web 上从 $BRANCH 发起 PR 到 main）"
  else
    # 无 remote → 本地 PR 模拟（单人项目自审可接受）
    echo "✅ 已 commit 到 $BRANCH 分支（本地无 remote）。"
    echo "请审核改动后，按 R6 在主 repo merge："
    echo "  git checkout $BASE_BRANCH"
    echo "  git merge --no-ff $BRANCH -m 'merge: $TITLE'"
    echo "  git branch -d $BRANCH"
  fi
  ```

### 完工总结

```markdown
## ✅ S2V 项目初始化完成 — 全套文档已就绪（Draft 状态）

> ⚠️ **重要**：本次生成的产物默认状态分三类：
> - **Phase spec / Task spec**：默认 `Status = Draft`（按 full-standard §10.5.1 状态机）；Draft 状态**允许 `<TBD-by-user>` 占位符**，但**禁止进入 `/s2v-implement` 或任何写代码的 agent**
> - **ADR**：默认 `Status = Accepted`（从 PRD §Decisions Log 转出即生效；ADR 自有状态机：Proposed / Accepted / Deprecated / Superseded）
> - **BDD feature**（.feature 文件）：占位场景文件，**不参与 §10.5.1 Spec Status 状态机**（场景内容由 task agent 实施时填充）
>
> **下一步必做**：审 task spec 业务字段（§3 Scope / §5 Behavior Contract / §6 AC / §7 追踪表），填完所有 `<TBD-by-user>`，把顶部 Status 字段从 `Draft` 改成 `Ready`，才能跑 `/s2v-implement <task>`。
> **每个 phase 还须**：在该 phase 最后一个 task 完工/合并前把 **phase spec §6（阶段级 AC + 端到端 smoke）填实**（init 时是 `<TBD-by-user>`）—— `s2v_preflight_phase` 会在 solo SOP 步 7 / team §4 Gate 3 强制，§6 仍 `<TBD>`/空 → BLOCK（C1 集成兜底，solo 档此为该 phase 唯一兜底）。§6 里**示例性尖括号语法**（如 `status <run>`、`<pipeline-id>`）是合法 spec 内容 —— 门只拦 S2V 保留占位 `<TBD-by-user>`/`<TBD-after-impl>`，示例无需改写绕开。

| 配置 | 值 |
|---|---|
| Project Name | <PROJECT_NAME> |
| Tier | <TIER> |
| PRD | <PRD_PATH> |
| Task 拆分模式 | AUTO / STEPWISE |
| **所有 spec Status** | **Draft（不可直接实施，需用户审核后改 Ready）** |

| 产物类别 | 数量 | 位置 |
|---|---|---|
| Adapter | 1 | docs/s2v-adapter.md |
| AGENTS.md | 1 | AGENTS.md |
| Phase Spec | N | docs/specs/phases/ |
| Task Spec | M | docs/specs/tasks/ |
| ADR | K | docs/decisions/ |
| BDD Feature | X | test/features/ |
| 目录骨架 | 5 个 | docs/specs/{phases,tasks}/, docs/decisions/, test/{features,fixtures}/ |

### 必守提醒

任何 tier 都必守 S2V 核心：
- TDD Iron Law（先写失败测试）
- §2.5 三段 commit 节律（RED → GREEN → REFACTOR）
- 每个 task 走 spec → BDD → TDD → Verification 链路
- 卡住时写 BLOCKED-task-X.Y.md 求助

### 下一步

1. **审核生成内容**：花 5-10 min 通读 adapter / AGENTS.md / phase spec 头部 / task spec 占位字段
1a. **⚠️ 审 adapter 剩余占位**（关键 — 否则 `/s2v-implement` 后续阶段可能 hard-fail / 路径不匹配）：
   步 5 已强制收集 5 个核心字段（基线绿三件套 `<INSTALL_COMMANDS>` / `<TYPECHECK_COMMANDS>` / `<UNIT_TEST_COMMANDS>` + Source/Test Areas 路径锚点 `<SOURCE_AREAS>` / `<UNIT_TEST_AREAS>`）；剩余 ~14 个占位（6 个 §Commands + 2 个 §Source Areas + 6 个 §Constraints）需在此审改：
   - **§Commands 剩余 6 项**（lint / integration / e2e / build / coverage / runtime-smoke）— 替换为真实命令（如 `pnpm lint` / `pnpm build`）或显式 `N/A: <原因>`（verify.sh 顶部"字段语义"注释块：空 / `N/A:<原因>` → 合法跳过）
   - **§Source Areas 剩余 2 项**（`<INTEGRATION_TEST_AREAS>` / `<E2E_TEST_AREAS>` — adapter §Source And Test Areas **弱约束**字段：implement / helper 当前不直接消费，故允许 `N/A`；区别于 §Source areas / §Unit test areas **强约束**禁 `N/A`）— 项目无对应测试时填 `N/A: <原因>`，否则给路径
   - **§Constraints 6 项** — 从 PRD §Technical Approach / §Constraints 派生 + 用户审改（verify.sh 不检查，但 implement.md / agents-*.md 会引用）
   - ⚠️ §Commands 剩余字段保留 `<...>` 占位且被 task §9 引用 → verify.sh hard-fail（"❌ adapter §Commands - `<field>` 仍是未替换占位"）
   - ⚠️ **§Commands 补填后须重跑一次步 5a 自检**：若此处把占位替换为真实命令，需对新填命令再次逐条检查是否与 §Constraints 冲突（步 5a 仅覆盖 init 时已填值，事后补填不受步 5a 自动保护）
2. **审核渲染产物**：
   - **模式 B 字段**（§3 Scope / §5.2 Imports / §5.3 函数签名 等）：填补 `<TBD-by-user>` 占位
   - **模式 A 字段**（§6 AC — agent 已从 PRD 推导渲染）：审核 agent 给的内容，如有偏差**修改而非填充**（不挂 `<TBD-by-user>` 前缀）
3. **开始实施 Phase 1**（task spec 改成 Status: Ready 后）：
```
   /s2v-implement docs/specs/tasks/task-1.1-<name>.md
   ```
   ⚠️ task spec 留在原地不归档（SDD 单一事实源核心要求）。
4. **后续如需追加产物**（罕见，如 v1.1 加新 phase / 突发 ADR）：
   ```
   /s2v-add <type> <name>
   ```
5. **后续如需调整协作模式**：
   ```
   /s2v-tier <new-tier>
   ```

### 参考（项目内自包含）

本次 init 已把 S2V 完整规范快照复制到 `docs/s2v/`，**优先看项目内**：

- 完整规范 · `docs/s2v/standard.md`
- Tier 决策树 · `docs/s2v/tier-decision-tree.md`（步 5.5 复制 — 规范快照前移以兜底 STEPWISE 中途取消；与 AGENTS.md / tier.md 引用一致）
- 模板归档（adapter / agents-<tier>）· `docs/s2v/templates-used/`

> 全局 skill 源文件位于 `${S2V_SKILL_DIR}`（步 0 解析；Claude Code 默认 `~/.claude/skills/s2v/`，其他 agent 见 [full-standard §22 Installation Paths by Agent](full-standard.md#22-installation-paths-by-agent)），**只在升级规范时改源文件**；项目内快照请勿直接编辑（会被下次 `/s2v-init` 或手动 `cp` 覆盖）。

---

## 错误处理

| 场景 | 行为 |
|---|---|
| 未在 git repo | 提示 git init 或允许跳过 |
| 已存在 adapter | 拒绝，建议 /s2v-tier 或 /s2v-add |
| 无 PRD | 拒绝，强制要求先跑 /s2v-prd |
| PRD 解析失败（格式异常 / 无 §Implementation Phases 表）| 报错指出缺失字段，建议用户补 PRD 或手动建 adapter |
| 用户 STEPWISE 模式中途取消 | 保留已生成产物（adapter / AGENTS.md / phase spec / 已审 task spec — **步 8 STEPWISE 每接受一个 task 已同步增量追加到 adapter Task / Phase 索引** / **规范快照已就绪 — 步 5.5 已复制 `docs/s2v/scripts/`**）+ 提示"剩余 task 可后续用 /s2v-add 补；/s2v-implement 也可直接对已审过的 task 跑（adapter 索引已含对应行 + scripts/ 已就绪 — 双重兜底）" |
| 模板文件缺失 | 报错指向 `${S2V_SKILL_DIR}/templates/` 重装（步 0 resolver 解析的实际安装路径）|

---

## 与其他命令的关系

| 命令 | 关系 |
|---|---|
| `/s2v-prd` | **强前置**（推荐，零依赖）：交互生成 PRD；本命令拒绝执行无 PRD 项目 |
| `/s2v-add` | **互补**：本命令一次性生成全套；`/s2v-add` 用于事后追加（v1.1 新增 phase / 突发 ADR）|
| `/s2v-tier` | **互斥**：tier 调整走它，不重新 init |
| `/s2v-implement` | **必然后续**：所有 S2V task spec 都走它实施（preflight Ready / 三段 commit / 回填 §10 / 不归档 spec）|

---

## 实施 agent 的硬约束

- ✅ 严格按定义的步号顺序（步 0 → 13），不跳过
- ✅ 每步完成后简短汇报进度
- ✅ 步 3 Tier 选择 / 步 4 Task 拆分模式 必须问用户，不要默认
- ✅ team 档**必须**走 chore branch + PR 流程（R6 自身适用）
- ✅ **模式 B（模板默认）字段**用 `<TBD-by-user>` 标占位（如 task §3 Scope / §5.2 Imports / §5.3 函数签名 等需用户业务决策的字段）
- ✅ **模式 A 字段**（task §6 AC — 详见 standard §8.3 §6 渲染规则 + 步 8 §6 AC 段）：从 PRD 推导后完整给值 + PRD 引用标注，由用户 review 时改；**不**挂 `<TBD-by-user>` 前缀
- ❌ 禁止默认套用某 tier — 必须问用户
- ❌ 禁止跳过 PRD 检查 — 强前置
- ❌ 禁止编造**模式 B 字段**（如 task §3 Scope / §5.2 Imports / §5.3 函数签名 等需要用户业务决策的字段）— 必须留 `<TBD-by-user>` 让用户填
- ❌ 禁止改全局 `${S2V_SKILL_DIR}/templates/` 内容（步 0 resolver 解析的实际安装路径）
