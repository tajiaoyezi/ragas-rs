---
name: s2v-implement
description: |
  端到端实施单个 S2V task spec，遵循 TDD 铁律（RED → GREEN → REFACTOR）、§2.5 三段 commit 节律、§9 Verification Plan 执行、§10 Completion Notes 回填。从项目 AGENTS.md 读取 tier 选择 git 流程（solo 直接在 main 上跑 / team 在 worktree + PR 上跑）。**若 task spec 是 Draft，先跑交互式审核（见 references/preflight-interactive-review.md），再自动把 Status 升到 Ready 并 commit。** **task spec 不归档** —— 留在 `docs/specs/tasks/` 作为 SDD 单一事实源（这是核心约束）。
when_to_use: |
  触发：`/s2v-implement <task-spec-path>`、"实施 task-X.Y"、"按 spec 实现 X"。跳过场景：输入不是 `docs/specs/tasks/` 下的路径。每次调用实施一个 task；整个 phase 的工作请按依赖顺序逐 task 调用。
origin: user
---

# /s2v-implement — S2V Task Spec 实施命令

> 读 S2V task spec → preflight Ready → 基线绿 → RED → GREEN → REFACTOR → §9 Verification → 回填 §10。
>
> **关键铁律**：task spec 是 SDD 单一事实源，实施完后**留在原地**不归档。
>
> tier-aware：solo 直接在 main 上跑三段 commit；team 在 feature branch + worktree 跑，最后开 PR。

## TL;DR

- **输入**：repo-relative 路径 `docs/specs/tasks/task-X.Y-<name>.md`
- **依赖**：项目内 `docs/s2v-adapter.md` + `AGENTS.md` + `docs/s2v/scripts/lib/`（由 `/s2v-init` / `/s2v-tier` 复制）
- **输出**：三段 commit + §10 回填 + §7 追踪表 Done + adapter Task 索引 Status=Done
- **失败兜底**：步 11.B 写 `BLOCKED-task-X.Y.md`（详见 [`references/blocked-protocol.md`](references/blocked-protocol.md)）

---

## 何时用

| 情况 | 命令 |
|---|---|
| 实施 S2V 项目里的某个 task spec | ✅ `/s2v-implement docs/specs/tasks/task-X.Y-<name>.md` |
| 想一次跑多个 task | ❌ 一次只能一个；多 task 由主 agent 按依赖顺序串行调度 |

---

## 执行流程（步 0 → 12）

### 步 0：输入 + 环境校验

> ⚠️ **本步起所有 helper 必须在 bash 下执行**。preflight.sh / verify.sh 顶部有 shell guard：
> 非 bash（zsh 等，**macOS Catalina+ 默认 shell 即 zsh**）下直接 `source` 会命中 guard 干净退出
> （`❌ S2V 脚本需在 bash 下运行…请改用 bash <脚本>`）—— 这是主力命令第一步，字面照做（默认 shell）即踩。
> agent 须显式用 `bash -c '...'` 包裹本块（或先进 bash 子 shell），不要在 zsh 里直接 source。

```bash
# 加载 helper（项目自包含路径，由 /s2v-init 复制到 docs/s2v/scripts/）
# ⚠️ 须 bash 执行（见上方提示）— 例：bash -c 'source docs/s2v/scripts/lib/preflight.sh; ...'
source docs/s2v/scripts/lib/preflight.sh
source docs/s2v/scripts/lib/verify.sh

TASK_SPEC="$ARGUMENTS"

# A. 输入路径形态（绝对路径 / ./ 前缀 / 非 docs/specs/tasks/ 路径都拒绝）
s2v_preflight_input "$TASK_SPEC" || exit 2

# B. 项目必须是 S2V 项目（adapter + AGENTS.md 都得有）
[ -f docs/s2v-adapter.md ] || { echo "❌ 当前目录无 docs/s2v-adapter.md - 先跑 /s2v-init"; exit 1; }
[ -f AGENTS.md ]            || { echo "❌ 缺少项目根 AGENTS.md（应由 /s2v-init 生成）"; exit 1; }

# C. 解析 TASK_ID / TASK_NAME（任何 tier 都用，不要只放在 team 分支里）
TASK_BASENAME=$(basename "$TASK_SPEC" .md)
TASK_ID=$(echo "$TASK_BASENAME" | sed -E 's/^task-([0-9.]+)-.*/\1/')
TASK_NAME=$(echo "$TASK_BASENAME" | sed -E 's/^task-[0-9.]+-//')
[ -z "$TASK_ID" ] || [ "$TASK_ID" = "$TASK_BASENAME" ] && {
  echo "❌ 无法从 $TASK_SPEC 解析 TASK_ID（要求文件名格式：task-<X.Y>-<name>.md）"
  exit 1
}
echo "✅ TASK_ID=${TASK_ID}  TASK_NAME=${TASK_NAME}  TASK_SPEC=${TASK_SPEC}"
```

### 步 1：读规格（按 §5.1 Required Reading 链路）

> ⚠️ 用 Read 工具按顺序读，不可跳过。读完才能动手。

按下表顺序加载全部上下文：

1. `AGENTS.md`（含 Collaboration Tier + task SOP + 三段 commit 节律）
2. `docs/s2v-adapter.md`（§Commands 表 + §Test 命名 + §Workflow Overrides）
3. `$TASK_SPEC`（本次要实施的 task）
4. `$TASK_SPEC` §5.1 Required Reading 列出的所有上游文件（上游 task spec / ADR / `.feature` 文件）
5. （如有）`docs/s2v/standard.md` §2.5 / §10.5.1 / §11 / §12 等关键章节

提取以下变量到内存（agent 用对话回报）：

| 变量 | 来源 | 用途 |
|---|---|---|
| `TIER` | `AGENTS.md` 顶部 `Collaboration Tier = ...` | 决定 git 流程 |
| `SCOPE` | task spec 文件名 / §1 推断（如 `parser` / `cli` / `auth`）| commit message scope |
| `AC_LIST` | task spec §6 Acceptance Criteria | RED 测试源 |
| `SCEN_TEST_MAP` | task spec §7 追踪表 | RED 测试编号 |
| `IMPORTS` | task spec §5.2 Imports | 步 7 实现引用 |
| `SIGNATURES` | task spec §5.3 函数签名 | 步 7 实现骨架 |

verification 字段（INSTALL_CMD / TYPECHECK_CMD / UNIT_TEST_CMD / ...）由 helper 自己读 adapter，不需要在此提取。

### 步 2：PREFLIGHT — Ready Gate

```bash
s2v_preflight_ready "$TASK_SPEC"
case $? in
  0)
    # Ready 或 In Progress，可直接动手
    STATUS=$(s2v_read_status "$TASK_SPEC")
    [ "$STATUS" = "Ready" ] && NEED_PROMOTE_TO_IN_PROGRESS=1 || NEED_PROMOTE_TO_IN_PROGRESS=0
    NEED_INTERACTIVE_REVIEW=0
    ;;
  1)
    # Status=Draft，进 §2A 交互式审核协议
    NEED_INTERACTIVE_REVIEW=1
    NEED_PROMOTE_TO_IN_PROGRESS=1
    ;;
  *)
    # 硬性 STOP（错误已写到 stderr）
    exit 2
    ;;
esac
```

### 步 2A：Draft 前置审核交互（仅 `NEED_INTERACTIVE_REVIEW=1` 触发）

完整对话协议见 [`references/preflight-interactive-review.md`](references/preflight-interactive-review.md)：四阶段 — 列 §6 AC review / 逐项处理 `<TBD-by-user>` / 改 `Status: Draft → Ready` / commit"业务承诺"。

任一阶段用户回答 `abort` 立即 `exit 1` 不改 spec。

完成后流程**继续到步 3 基线绿**。步 5 会把 `Ready → In Progress`（不需要重跑 §2A）。

### 步 3：基线绿（动手前确认遗留无红）

```bash
# 单一实现见 scripts/lib/verify.sh s2v_baseline_green（冷启动判定**先于**任何门禁：
# greenfield 则跳过 install+typecheck+unit-test；否则 install→typecheck→unit-test。
# 判据见下方 blockquote）。<UNIT_TEST_AREAS>：读 adapter §Source And Test Areas >
# Unit test areas bullet list，每行一个 pathspec，空格分隔展开为多参数（无外层引号）。
s2v_baseline_green "<UNIT_TEST_AREAS>" || exit 1
```

> **冷启动豁免（全新项目首个 task）由 helper 自动判定**：greenfield 时 `install`（npm ci /
> pip -r 无 manifest）、`typecheck`（go vet ./... 零包）、`unit-test`（go test ./... 零包）
> 都必然非零但非真红。判据**先于 install 执行**且为**排除式 + 安全偏置**：areas 全部存在，
> 且 prune 依赖/构建/docs 目录后**无任何"非脚手架"文件**（脚手架 denylist 永不含源码扩展名）
> → 自动**跳过基线 install + typecheck + unit-test** 并在 §10 备注；任意未知/源码文件
> （含 .vue/.hs/.dart 等任意语言、Rust 内嵌测试源、编译型 §5.3 骨架）= 真实内容 → 三者正常强制（真红绝不被掩盖）。
> 若基线红是因为本 task 要解决的红测试（罕见，通常是 bug fix task），允许跳过这步并在 §10 备注。
> 若某条命令在 adapter 里**真的不适用**（项目无 typecheck 等），写 `N/A: <原因>` 而非留空，
> helper 会优雅跳过并在 §10 Verification 结果里如实记录"未运行原因"。

### 步 4：tier-aware git 准备

#### 4.A · solo

```bash
# 直接在 main 上工作，确认无未提交改动
[ -n "$(git status --porcelain)" ] && {
  echo "⚠️ working tree 有未提交改动 - 先 commit 或 stash 再实施"
  git status --short
  exit 1
}
EXPECTED_BRANCH=$(git branch --show-current)   # 通常是 main
```

#### 4.B · team

```bash
# 0. 主 repo 必须 clean —— 否则用户在主 repo 对 task spec 的「Draft→Ready 审核」
#    改动若未提交，git worktree add 取的是已提交 HEAD（旧 Draft），审核内容不会
#    进入 worktree → 步 2 preflight（在主 repo 校验的是含审核改动的文件）与步 6+
#    实际实施的不是同一份（破坏单一事实源）。对齐 solo 4.A 的脏树守卫，可执行而非仅注释。
if [ -n "$(git status --porcelain)" ]; then
  echo "⚠️ 主 repo working tree 有未提交改动 —— 先 commit（尤其是刚审过、Status 改 Ready 的 task spec）"
  echo "   或 git stash 再开 worktree，否则审核内容不会进入 worktree。"
  git status --short
  exit 1
fi

# 1. 创建 feature branch + worktree（此时主 repo 已 clean，审核改动已在 HEAD → 随分支进入 worktree）
BRANCH="feat/task-${TASK_ID}-${TASK_NAME}"
WORKTREE_PATH="../$(basename "$PWD")-wt-task-${TASK_ID}"

git branch "$BRANCH" 2>/dev/null || true
git worktree add "$WORKTREE_PATH" "$BRANCH"
cd "$WORKTREE_PATH"
EXPECTED_BRANCH="$BRANCH"

echo "✅ 已创建 worktree: ${WORKTREE_PATH} (branch=${BRANCH})"
echo "   后续步骤都在此 worktree 内执行"
```

### 步 5：把 Status 提升到 In Progress（如果还是 Ready）

```bash
if [ "$NEED_PROMOTE_TO_IN_PROGRESS" = "1" ]; then
  # portable perl（BSD/macOS sed 不支持 0,/.../ 范围地址；CI/Linux 容器 sed -i '' 又不支持）
  # 推荐 agent 直接用 Edit 工具改第一处 `**Status**: Ready` → `**Status**: In Progress`
  perl -i -pe 's/^\*\*Status\*\*:.*$/\*\*Status\*\*: In Progress/ if /^\*\*Status\*\*:/ && !$done++' "$TASK_SPEC"

  git add "$TASK_SPEC"
  git commit -m "docs(spec): task-${TASK_ID} 进入实施 (Status: Ready → In Progress)" | tee /tmp/c.txt
  grep -qE "^\[${EXPECTED_BRANCH} " /tmp/c.txt || { echo "BRANCH MISMATCH"; exit 1; }
fi
```

### 步 6：RED — 按 §6 AC 写失败测试

> 按 task spec §7 追踪表的 SCEN/TEST 编号 + adapter §Test 命名规则，把每个 AC 翻译成一个失败测试。

1. 用 Write/Edit 工具在 adapter §Source And Test Areas > Unit test areas 列出的路径下创建测试文件（按 adapter 命名约定；多 path 时按 module 归属择一）
2. 每个 AC 至少一个测试，编号对应 §7 追踪表（如 `SCEN-2.1.1` / `TEST-2.1.1`）
2a. **编译型语言（Java/Go/Rust/Kotlin 等）必做桥接**（解释型语言如 Python 跳过）：同时用 Write 工具在 §Source areas 下创建 task spec §5.3 的**可编译空骨架**（签名存在、体内 `throw UnsupportedOperationException` / 返回错值；无异常语言如 Go/Rust 用刻意 `panic("unimplemented")` / `unimplemented!()`，优先净断言或刻意显式 panic，勿留骨架返回零值导致的*附带* nil-panic/超时 —— 详见 §2.5.1），否则测试引用不存在的类型 = 编译失败 = 违 full-standard §2.5.1「不是语法错/找不到文件」、非合法 RED。骨架须使测试**因断言失败而红**而非编译失败。详见 full-standard §2.5.1「编译型语言 RED 桥接」。
3. 跑测试确认全部 RED（编译型确认为"功能未实现型断言失败"，非编译错）
4. commit + R3 校验

```bash
s2v_run unit-test 2>&1 | tee /tmp/red.txt || true
# 期望全部 fail；如果误绿（实现已存在），删测试或重写测试更严格

# agent 替换 <UNIT_TEST_AREAS>：读 adapter §Source And Test Areas > Unit test areas bullet list，
# 每行一个 pathspec，展开为多个 git add 参数（无外层引号 — list 直接喂给 git add）
#
# ⚠️ RED commit 必须自洽可复现：单独 checkout 该 commit 也能跑出"真红"。故 git add 范围
#    不止 <UNIT_TEST_AREAS>，还须含使红测试可复现的最小集：
#      ① 编译型语言：步 2a 的 §5.3 可编译骨架（在 <SOURCE_AREAS> 下）
#      ② 构建配置文件：pom.xml / build.gradle / Cargo.toml / go.mod / package.json 等
#         —— 既非 source 也非 test、adapter 无对应 §area，但缺它编译型项目 RED 无法复现；
#         首个 task 由本 task 创建构建系统时尤其必须一并 add
#    解释型无构建配置的项目（纯 Python 等）通常仅 <UNIT_TEST_AREAS> 即可。
git add <UNIT_TEST_AREAS>   # + 编译型再加：<SOURCE_AREAS> 下的 §5.3 骨架 + 构建配置文件（pom.xml 等）
# 静默丢失兜底（claim-3a / C10 源码版）：git add 对被 .gitignore 命中的路径**静默跳过**。
# 声明的 SOURCE/UNIT area 若被过宽 ignore 规则遮蔽（典型：裸二进制名吞 Go cmd/<bin>/），
# RED 会丢源码 / 假绿且无提示。复用 §0 已 source 的 preflight.sh（rc1=被遮蔽 STOP，
# rc2=参数/环境错；空格分隔多 pathspec，无外层引号）：
s2v_guard_areas_tracked <UNIT_TEST_AREAS> || { echo "❌ 声明 area 被 .gitignore 静默遮蔽/环境错 — 解决后重跑 RED（勿带丢源码的假 RED 入库）"; exit 1; }   # 编译型：s2v_guard_areas_tracked <UNIT_TEST_AREAS> <SOURCE_AREAS>
git commit -m "test(${SCOPE}): 加 SCEN-${TASK_ID}.1~${TASK_ID}.${AC_COUNT} 共 ${AC_COUNT} 个 RED 测试" | tee /tmp/c.txt
grep -qE "^\[${EXPECTED_BRANCH} " /tmp/c.txt || { echo "BRANCH MISMATCH"; exit 1; }
echo "✅ RED commit 完成"
```

### 步 7：GREEN — 写最小实现

> 按 task spec §5.2 Imports + §5.3 函数签名 写最简实现。**不实现 §3 Out-of-Scope 的内容**。

1. 用 Write/Edit 工具在 adapter §Source And Test Areas > Source areas 列出的路径下创建/修改源码
2. 按 §5.3 签名实现，不增加未在 §6 AC 涵盖的字段或方法
3. 跑测试确认全部 GREEN
4. commit + R3 校验

```bash
s2v_run typecheck || { echo "❌ typecheck 红"; exit 1; }
s2v_run unit-test 2>&1 | tee /tmp/green.txt
[ "${PIPESTATUS[0]}" -eq 0 ] || { echo "❌ unit-test 失败"; exit 1; }

# agent 替换 <SOURCE_AREAS>：读 adapter §Source And Test Areas > Source areas bullet list，
# 每行一个 pathspec，展开为多个 git add 参数（无外层引号）
git add <SOURCE_AREAS>
git commit -m "feat(${SCOPE}): 实现 <模块描述> 通过全部 ${AC_COUNT} 个测试" | tee /tmp/c.txt
grep -qE "^\[${EXPECTED_BRANCH} " /tmp/c.txt || { echo "BRANCH MISMATCH"; exit 1; }
echo "✅ GREEN commit 完成"
```

### 步 8：REFACTOR（可选）

如有重复代码 / 过长函数 / 命名不清，做最小重构。**测试必须保持绿**。

```bash
REFACTOR_BASE=$(git rev-parse HEAD)   # 记录 refactor 前 HEAD（用于精确回滚 refactor touched files）

# ... 进行 refactor ...

if ! s2v_run unit-test; then
  echo "❌ refactor 破坏测试。"
  echo "  本次 refactor 改动如下（不会自动回滚 — 请用户决策）："
  git diff --stat
  echo ""
  echo "  下一步选项："
  echo "  [A] 回滚仅 refactor touched files: git checkout ${REFACTOR_BASE} -- <files>"
  echo "  [B] 修复 refactor 让测试重新过"
  echo "  [C] 放弃 refactor 整段: git stash push -m 'failed refactor' -- <files>"
  echo "  🚫 禁止 git restore . 或 git reset --hard（会丢未跟踪改动 / 用户同步写入的内容）"
  exit 1
fi

git add <changed-files>   # agent 替换为本次 refactor 真实改动文件列表（多文件空格分隔为多参数，无外层引号；若单文件路径含空格则 agent 自行加引号）
git commit -m "refactor(${SCOPE}): <重构内容简述>" | tee /tmp/c.txt
grep -qE "^\[${EXPECTED_BRANCH} " /tmp/c.txt || { echo "BRANCH MISMATCH"; exit 1; }
echo "✅ REFACTOR commit 完成"
```

> 没有重构需求就跳过，不强制为重构而重构。

### 步 9：跑 §9 Verification Plan 全套

```bash
# helper 从 task spec §9 抽 key（按固定执行序，自动跳过非标行 + 警告字段名拼错）
VERIFY_KEYS=$(s2v_extract_verify_keys "$TASK_SPEC")

# 跑全套（空列表 / 缺 unit-test 自动 hard-fail）
s2v_verify_full "$VERIFY_KEYS" || {
  export FAIL_REASON="§9 Verification 失败"
  # 进卡住协议（步 11.B），完整模板见 references/blocked-protocol.md
  exit 11
}

# C4：覆盖率阈值契约门 —— task 声明了阈值但 adapter Coverage 命令不自我强制 → STOP
#     （verify.sh 只看 rc，不焊阈值则实测 < 阈值 也 rc0，绿灯不可信）
s2v_coverage_threshold_guard "$TASK_SPEC" || { export FAIL_REASON="覆盖率阈值未被强制"; exit 11; }
```

任一项失败 → 进卡住协议（步 11.B），写 `BLOCKED-task-X.Y.md` 求助；不要 skip 也不要伪造通过。

### 步 10：回填 task spec §10 Completion Notes

> ⚠️ §10 字段名 schema 的**唯一权威**是 `full-standard.md §8.3` 中的 Task Spec 模板。team merge gate 4 / CI 按这 6 个名字 grep。
> 💡 **C7**：可先跑 `s2v_backfill_notes "$TASK_SPEC"` 生成骨架 —— 自动填正确日期 + 与本 task §9 **1:1** 的结果行（杜绝 Gate 4 第 1.5 道 BLOCK），其余业务字段按下方模板填实再 commit。

用 Edit 工具在 task spec §10 段落填入：

```markdown
## 10. Completion Notes

- **完成日期**：YYYY-MM-DD
- **改动文件**：
  - `<source-file-1>`（新增/修改）
  - `<source-file-2>`
  - `<test-file-1>`
- **commit 列表**：
  - `<hash1>` test: 加 RED 测试
  - `<hash2>` feat: 实现
  - `<hash3>` refactor:（如有）
- **§9 Verification 结果**：（按本 task §9 实际执行的项逐项记录；§9 没列的删除该行）
  - install: ✅ / skipped: <reason> / N/A
  - lint: ✅ / skipped: <reason> / N/A
  - typecheck: ✅ / skipped: <reason> / N/A
  - unit-test: N passed / 0 failed   <!-- 强制：unit-test 不允许 skipped -->
  - integration: ✅ / skipped: <reason> / N/A
  - e2e: ✅ / skipped: <reason> / N/A
  - build: ✅ / skipped: <reason> / N/A
  - coverage: NN.N% / 阈值 NN%
  - runtime-smoke: ✅ <evidence: 端口/截图/日志> / skipped: <reason>
  - manual: ✅ <证据/截图/确认者> / N/A: <reason>
- **剩余风险 / 未做项**：`<RISK_OR_NONE>`
- **下游 task 影响**：`<DOWNSTREAM_OR_NONE>`

<!-- 条件性第 7 项：若本 task 含任何 Waived AC（顶部 Status=Waived 或 §7 追踪表有 Waived 行），
     在此追加「Waiver 登记」段，按 standard §12.3 五项 schema 填：
       - **Waiver 登记**：
         - 豁免对象：<AC ID>
         - 原因：<一句话>
         - 替代验证：<怎么核验>
         - 补齐条件：<何时/什么条件下补>
         - 负责人：<owner>
     完整模板见 full-standard.md §8.3 §10 末段。漏填会被 team Gate 4 第 4 道 BLOCK。
     ⚠️ Waive 不可达 AC 的 task 多为「从未实施」：除追加 Waiver 登记外，**必须同时规范化上面 6 项**
        —— 完成日期填 Waive 当天日期；改动文件/commit/§9结果/剩余风险/下游 未实施填字面量
        `无（已 Waive，未实施）`；清除所有 <TBD-after-impl>。否则 team Gate 4 door 1（缺完成日期）/
        door 2（占位拒绝）会**先于** Waiver 专用 door BLOCK，文档化 Waive 路径无法 merge。 -->
```

⚠️ **占位必须全部替换为真实值再 commit**：

- `YYYY-MM-DD` → 真实日期
- `<source-file-N>` / `<test-file-N>` → 真实文件相对路径
- `<hash1>` / `<hash2>` / `<hash3>` → `git log` 拿到的真实 short hash
- `<RISK_OR_NONE>` → 一句话风险描述，或字面量 `无`
- `<DOWNSTREAM_OR_NONE>` → 影响哪些下游 task ID 列表，或字面量 `无`

team merge gate 4 / CI 不仅检查字段名是否齐全，还会 grep `§10` 段落里**任何 `<XXX>` 形式的模板 token**，发现即 BLOCK（防止"字段在但值仍是占位"通过）。

同时更新顶部 Status 字段：In Progress → Done。

并把 §7 追踪表里本 task 的**所有非终态行**（`Not Started` / `Spec Ready` / `Scenario Ready` / `Test Red` / `In Progress` / `Verified` — 即 full-standard §12.2 行级状态机中除 `Done` / `Waived` / `Blocked` 外的全部状态）改成 `Done`；**`Waived` / `Blocked` 行必须保留原状态**（行级 `Waived` 是 team Gate 4 第 4 道触发审计登记的依据，覆盖会让 Waiver 检查失效 → 审计链断裂）。⚠️ init 阶段所有 §7 行默认 `Not Started`（见 init.md 步 8），实施全程未必逐行推进中间态 — 完工时必须覆盖 `Not Started` 等全部非终态，与 team Gate 4 第 3.5 道（顶部 `Done` 时拦截一切非 `Done`/`Waived`/`Blocked` 行）拦截集合一致，否则严格按本步执行的 task 会被 Gate BLOCK。

### 步 11：commit §10 回填 + 完工

#### 11.A · 正常完工（步 9 全过）

> **C8**：本 canonical 路径已被步 9 的 `s2v_verify_full ... || exit 11` 隐式绑定
> （§9 红则到不了这里）。外部 / 手动 agent 若不照本流程跑、想直接把 Status 改
> `Done`，**必须先 `s2v_require_green "$TASK_SPEC"`**（等同 team Gate 2，Done 不可自证）。

```bash
git add "$TASK_SPEC"
git commit -m "docs(spec): 回填 task-${TASK_ID} §10 Completion Notes (Status: Done)" | tee /tmp/c.txt
grep -qE "^\[${EXPECTED_BRANCH} " /tmp/c.txt || { echo "BRANCH MISMATCH"; exit 1; }

# 同步更新 adapter 的 Task 总索引（§Status 列）
# 用 Edit 工具把 docs/s2v-adapter.md 里本 task 的 Status 改成 Done
git add docs/s2v-adapter.md
git commit -m "docs(adapter): 标记 task-${TASK_ID} 为 Done" | tee /tmp/c.txt
grep -qE "^\[${EXPECTED_BRANCH} " /tmp/c.txt || { echo "BRANCH MISMATCH"; exit 1; }
```

#### 11.B · 卡住协议

走完整 BLOCKED-task-X.Y.md 模板：见 [`references/blocked-protocol.md`](references/blocked-protocol.md)。
写完 BLOCKED 文档后 commit + 退出实施，等用户 / 主 agent 决策。

### 步 12：tier-aware 收尾

#### 12.A · solo

直接完工，对话回报：

```text
✅ task-${TASK_ID} 已实施完成
- 三段 commit + §10 回填，全部在 main 上
- §9 Verification 全过
- 下一步：跑 /s2v-implement 实施下一个 task
```

> **C1 Phase 兜底**：若本 task 是其所属 phase 的最后一个 task，完工后必须按 AGENTS.md
> solo SOP 步 7 跑 `s2v_preflight_phase <phase-spec>` + phase §6 端到端 smoke（§6 仍
> `<TBD>`/空 → STOP，不算 phase 完成）。solo 档无 merge gate，这是该 phase 唯一集成兜底。

#### 12.B · team

提示用户开 PR：

```text
✅ task-${TASK_ID} 已在 worktree 实施完成
- worktree: ${WORKTREE_PATH}
- branch: ${BRANCH}
- 三段 commit + §10 回填 + adapter Status 同步

下一步（在 worktree 内）：
  cd ${WORKTREE_PATH}
  # 有 remote：
  git push -u origin ${BRANCH}
  gh pr create --base main --title "task-${TASK_ID}: <一句话>" \
    --body "实现 task-${TASK_ID}，详见 docs/specs/tasks/task-${TASK_ID}-*.md"
  #（若 gh 未安装：在 GitHub/GitLab Web 上从 ${BRANCH} 发起 PR 到 main）
  # 无 remote：写 READY-FOR-MERGE-task-${TASK_ID}.md + commit，通知主 agent 在主 repo 按 AGENTS.md R6.1 本地 merge --no-ff

PR 合并由主 agent 在主 repo 跑 AGENTS.md §4 gate 流程完成：
  - worktree 由 §4 Gate 0 自动回收（其 commit 已在 feature branch 上，不丢提交）
  - feature branch 由 Gate 5 之后清理
  task agent / 用户**无需**再手动 git worktree remove（避免与 Gate 0 重复 remove）
```

---

## 错误处理

| 场景 | 行为 |
|---|---|
| 缺参数 / 文件不存在 / 路径形态错 | `s2v_preflight_input` 报错 + 用法提示 |
| 项目无 `docs/s2v-adapter.md` | 拒绝 + 建议先 `/s2v-init` |
| Status=Draft | 进入 §2A 前置审核交互 — 用户交互后自动 Draft → Ready + commit |
| Status=Ready 但仍有 `<TBD-by-user>` | STOP（用户漏填，要求清零再重跑）|
| Status=Done / Blocked / Waived | STOP + 提示先解决状态 |
| §6 AC 列表空 / §7 追踪表空 | STOP + 指出 spec 不完整 |
| 基线红 | STOP + 提示先解决遗留 |
| AC 反复失败 ≥3 次 | 走步 11.B 卡住协议（[`references/blocked-protocol.md`](references/blocked-protocol.md)）|
| §9 Verification 任一失败 | 走步 11.B 卡住协议 |
| commit 后 R3 校验 branch mismatch | 走 AGENTS.md §5 场景 A 安全修复（不直接 reset）|

---

## 与其他命令的关系

| 命令 | 关系 |
|---|---|
| `/s2v-prd` | 远前置（生成项目级 PRD）|
| `/s2v-init` | **强前置**：必须有 adapter 才能跑本命令 |
| `/s2v-add` | **互补**：用于追加新 task spec；本命令实施已有 task |
| `/s2v-tier` | **正交**：tier 调整后本命令读新 AGENTS.md 自动适配 |

---

## 实施 agent 的硬约束

- ✅ 严格按定义的步号顺序（步 0 → 12），不跳步
- ✅ 每步完成简短汇报（"步 N/12：..."）
- ✅ 每个 commit 后立即跑 R3 [branch] 校验
- ✅ 严格按 task spec §6 AC 写测试（不擅自加未在 AC 的测试）
- ✅ 严格按 task spec §5 实现（不擅自加未在 §5.3 签名的字段/方法）
- ✅ §9 Verification 必须全过才能进步 10
- ❌ **禁止移动或归档 task spec**（SDD 单一事实源核心要求）
- ❌ 禁止跳过 RED 直接 GREEN（违反 TDD Iron Law）
- ❌ 禁止把 §9 任一失败说成"通过"
- ❌ 禁止改 task spec §6 AC（业务字段，要改先 STOP 让用户审）
- ❌ 禁止改全局 skill 目录（实际路径见 `docs/s2v/standard.md` §22；Claude Code 默认 `~/.claude/skills/s2v/`）
- ❌ 禁止在 team tier 下不开 worktree 直接在主 repo 写代码
- ❌ 禁止把 commit 落到 wrong branch 后用 `git reset --hard` 修复（走 AGENTS.md §5 场景 A）
