# AGENTS.md — 协作约定（Tier: solo）

> 本项目 Collaboration Tier = `solo`。单人 / 单 agent / 快速迭代场景。
> git 协作宽松：直接在 main 上工作，不需要 worktree / feature branch / PR 流程。
> **但 S2V 开发规范的核心方法论仍 100% 必守**——见下方"必守清单"。
>
> **任何 agent（含外部 Codex / Cursor / Aider）进入本仓库时第一件事：读完本文件 + `<adapter-path>`**。

---

## 必守清单（任何 tier 都不可降级）

S2V 开发规范的以下 8 项由所有 tier 共同强制，`solo` 不是"跳过测试 / 跳过 spec / 跳过 ADR"的免责证书：

1. **SDD**：master spec / phase spec / task spec 必写（用 `/s2v-add` 命令生成）
2. **BDD**：`.feature` 文件按需写（用户可见行为必有）
3. **TDD Iron Law**：先写失败测试，再写实现 — 没有 RED 的 commit 禁止 GREEN
4. **§2.5 三段 commit 节律**：每个 task 至少 RED commit + GREEN commit（REFACTOR 可选）
   - solo 档下三段 commit 都直接落 main，仍是 TDD 履迹的可验证性，不可省略
5. **ADR**：架构 / 依赖 / 协议 / 安全 / 数据决策必写
6. **Verification**：每个 task done 必跑 task §9 实际列出的全部验证项（`s2v_verify_full "$(s2v_extract_verify_keys "$TASK_SPEC")"`；unit-test 强制，typecheck / coverage 等按 §9 与 adapter §Commands 非 N/A 实际列入 — 不硬编码也不漏跑）
7. **§7 追踪表**：每个 task spec §7 维护 AC ↔ SCEN ↔ TEST 映射
8. **卡住协议**：AC 失败 ≥3 次后写 `BLOCKED-task-<X.Y>.md`（仍要走，单人时即写给"未来的自己"看）

---

## S2V 命令派发（任何进入本项目的 agent 必读）

本项目已初始化 S2V。当用户输入 `/s2v-implement` / `/s2v-add` / `/s2v-tier`
（或用自然语言要求该子流程）：即便你的工具未注册此 `/` 命令、它只是一条普通消息 ——
你已加载本 `AGENTS.md`，据此执行（命令后的文本即其参数）：

- `/s2v-implement <task-spec 路径>` → 严格执行本文件 **task 启动 5 步 SOP**
  （项目自包含，无需解析 skill 目录）
- `/s2v-add <类型> <名称>` / `/s2v-tier <目标档>` → 解析 s2v skill 目录
  （`$S2V_SKILL_DIR` 或 `full-standard.md` §22.2 已知默认路径）→ 读并严格执行
  `<skill>/add.md` ｜ `<skill>/tier.md`
- `/s2v-prd` / `/s2v-init` 为建项 / 重建项命令，通常不在已初始化项目内运行；确需时走 skill 目录对应文档

Aider 等无 slash 命令系统的工具：以 `--read AGENTS.md` 加载本文件后本规则同样适用。

---

## 0. Adapter 命令读取规则（同 team 档；solo / team 共用同一份 helper）

下面所有 baseline / verification 命令调用都依赖以下 helper。任何 agent 进入仓库后**先 source 一次本块**再执行下方"task 启动 5 步 SOP"。

> ⚠️ **必须在 bash 下 source**。helper 顶部有 shell guard：非 bash（zsh 等，macOS Catalina+ 默认 shell 即 zsh）下直接 `source` 会命中 guard 干净退出（提示改用 bash）。agent 须显式 `bash -c '...'` 包裹本块及后续 SOP 调用，不要在 zsh 里直接 source。

```bash
# 项目内自包含 helper（由 /s2v-init 步 5.5 / /s2v-tier 步 2.5 同步刷新）
# ⚠️ 须 bash 执行：bash -c 'source docs/s2v/scripts/lib/preflight.sh; source docs/s2v/scripts/lib/verify.sh; ...'
source docs/s2v/scripts/lib/preflight.sh
source docs/s2v/scripts/lib/verify.sh
```

提供的函数：

| 函数 | 用途 |
|---|---|
| `s2v_load_cmd <字段>` | 取 adapter §Commands 字段值（Install / Lint / Typecheck / Unit Test / Integration tests / E2E tests / Coverage / Build / Runtime smoke）|
| `s2v_run <key> [required]` | 执行 verification key；占位 hard-fail / N/A 跳过 / unit-test 自动 required |
| `s2v_extract_verify_keys <task-spec>` | 从 task spec §9 抽 key（按固定安全执行序）|
| `s2v_verify_full "<keys>"` | 跑全套；空列表 / 缺 unit-test 自动 hard-fail |
| `s2v_read_status <task-spec>` | 读 task spec 顶部 `**Status**:` 字段（多词如 "In Progress" 不被截断）|
| `s2v_preflight_input <path>` | 输入路径形态校验（绝对路径 / ./ 前缀 / 非 docs/specs/tasks/ 拒绝）|
| `s2v_preflight_ready <task-spec>` | Ready Gate 全套；rc=0 OK / rc=1 Draft（本档 SOP 步 2.5：STOP，让用户自审改 Ready）/ rc=2 硬 STOP |

完整说明 + self-test：见 `docs/s2v/scripts/README.md`。solo / team 共用同一份语义。

> ⚠️ 不要硬编码 4 项验证 — 按 task spec §9 实际列出的字段调用 `s2v_verify_full "$(s2v_extract_verify_keys "$TASK_SPEC")"`。
>
> **修改 helper 行为**：改全局 skill 的 `scripts/lib/`（路径见 `docs/s2v/standard.md` §22；Claude Code 默认 `~/.claude/skills/s2v/scripts/lib/`，其他 agent 见 §22），跑 `bash scripts/lib/_self-test.sh` 验证，再用 `/s2v-tier` 把项目内快照刷新到最新。**不要直接编辑 `docs/s2v/scripts/`**（会被覆盖）。

---

## task 启动 5 步 SOP（每个 task 开始前必走）

```bash
# 1. 基线绿（确认动手前没有遗留红测试）
#    单一实现 = scripts/lib/verify.sh s2v_baseline_green（冷启动判定先于门禁：
#    greenfield 跳过 install+typecheck+unit-test，否则 install→typecheck→unit-test；
#    判据=排除式+安全偏置：prune 依赖/docs 后无非脚手架文件=greenfield，非白名单、
#    不靠退出码）。<UNIT_TEST_AREAS>：读 adapter §Source And Test Areas > Unit test areas bullet list，空格分隔多 pathspec（无外层引号）。
s2v_baseline_green "<UNIT_TEST_AREAS>" || exit 1
# 失败 → 先修复或确认是当前 task 要解决的红，否则不开干

# 2. 读规格（按顺序，不可跳过）
#    a. AGENTS.md（本文件）
#    b. <adapter-path>
#    c. <task-spec-path>（本次要实施的 task）
#    d. 该 spec §5.1 Required Reading 列出的所有上游 spec
#    e. 对应 .feature 文件
#    f. 相关 ADR

# 2.5. PREFLIGHT — Ready Gate（不通过禁止进 RED）
#      复用 §0 已 source 的 preflight.sh（与 /s2v-implement 步 2 同一 Ready Gate —
#      含 Status 多词解析 / <TBD-by-user> / §6 AC 非空 / §7 SCEN-TEST 非空 全套检查；
#      手写 inline 会漏 §6/§7 空检查，已统一改用 helper）
TASK_SPEC="<task-spec-path>"   # agent 替换为真实 task spec 路径
s2v_preflight_ready "$TASK_SPEC"
case $? in
  0) : ;;                        # Ready / In Progress，可进 RED
  1) echo "🛑 STOP: $TASK_SPEC Status=Draft — solo 档先自审 §3 Scope / §5 Behavior Contract / §6 AC，把 Status 改成 Ready 再来"
     exit 1 ;;
  *) exit 2 ;;                   # 硬性 STOP（§6 AC 空 / §7 无 SCEN-TEST / 非法 Status / 残留 <TBD-by-user> — 详因已写 stderr）
esac

# 3. RED → GREEN → REFACTOR 三段 commit（按 §2.5 commit 节律）
#    每段 commit 后立即跑 R3 [branch] 校验（见下）

# 4. task done 前跑 task §9 Verification Plan 全套
#    §0 helper 自动从 task §9 抽取 key（不要硬编码清单）
TASK_SPEC="<task-spec-path>"   # agent 替换为真实 task spec 路径
VERIFY_KEYS="$(s2v_extract_verify_keys "$TASK_SPEC")"
s2v_verify_full "$VERIFY_KEYS" || exit 1
# C4：覆盖率阈值契约门（声明阈值但 Coverage 命令不自我强制 → STOP）
s2v_coverage_threshold_guard "$TASK_SPEC" || exit 1
# 全过 → 进步 5；任一失败 → 修复或走卡住协议

# 5. 回填 task spec §10 Completion Notes
#    必须按 docs/s2v/standard.md §8.3 的 6 项中文 schema 全填，不要省略：
#      - 完成日期（YYYY-MM-DD）
#      - 改动文件
#      - commit 列表（hash + message）
#      - §9 Verification 结果（按本 task §9 实际列出的 key 逐行展开：install/lint/typecheck/unit-test/integration/e2e/build/coverage/runtime-smoke/manual）
#      - 剩余风险 / 未做项
#      - 下游 task 影响
#    缺任一项 = team merge gate / CI 检查器会 BLOCKED；solo 档无 gate，但 schema 必须一致
#    （solo 升档到 team 后旧记录不会再被修一遍）

# 5.1. self-check（solo 档：不 BLOCK，只 warn — 因为 solo 没有 merge gate，但留 audit trail）
#      与 team Gate 4 第 1 道用同一组 grep；让 LLM agent 至少看见缺项告警。
NOTES=$(awk '
  /^## 10\. Completion Notes/ { in_section=1; next }
  in_section && /^## /        { in_section=0 }
  in_section                  { print }
' "$TASK_SPEC")
MISSING=""
echo "$NOTES" | grep -qE "完成日期.*20[0-9]{2}-[0-9]{2}-[0-9]{2}" || MISSING="${MISSING} 完成日期"
echo "$NOTES" | grep -qE "改动文件"                                  || MISSING="${MISSING} 改动文件"
echo "$NOTES" | grep -qE "commit 列表"                               || MISSING="${MISSING} commit列表"
echo "$NOTES" | grep -qE "Verification 结果|verification 结果"       || MISSING="${MISSING} Verification结果"
echo "$NOTES" | grep -qE "剩余风险"                                  || MISSING="${MISSING} 剩余风险"
echo "$NOTES" | grep -qE "下游 task 影响|下游影响"                    || MISSING="${MISSING} 下游影响"
if [ -n "$MISSING" ]; then
  echo "⚠️  §10 Completion Notes 缺以下字段（solo 档不 BLOCK，但建议补齐 — 升档到 team 后会被 Gate 4 拒）：${MISSING}"
fi

# 占位检查（warn-only，与 team Gate 4 第 2 道同逻辑）— 未替换的 <XXX> 会让升档后 BLOCK。
# 先过 _s2v_strip_retained（§0 已 source preflight.sh，单一源剥除函数）：剥掉
# §8.3 / §10 模板保留的 <!-- --> 注释 / ^> blockquote —— 否则 §10 schema 指引
# blockquote 里的合法字面 <TBD-after-impl> 会被误判为"未替换占位"，正确完工的
# task 升档后被 Gate 4 误 BLOCK（DEFECT-P3-C，与 DEFECT-1 同根）。
# ⚠️ 禁止改回裸 grep — 剥除逻辑须保持单一源，见 preflight.sh _S2V_STRIP_PREAMBLE。
PLACEHOLDERS=$(echo "$NOTES" | _s2v_strip_retained | grep -oE "<[A-Za-z_][A-Za-z0-9_-]*>" | sort -u || true)
if [ -n "$PLACEHOLDERS" ]; then
  echo "⚠️  §10 仍含未替换的占位（升档到 team 后 Gate 4 第 2 道会 BLOCK）："
  echo "    ↳ 若本 task 是 Waive 未实施：未实施字段填『无（已 Waive，未实施）』并清占位（同 team『Waive 后的留痕要求』）"
  echo "$PLACEHOLDERS" | sed 's/^/    - /'
fi

# §10 keys vs §9 keys 集合检查（warn-only，与 team Gate 4 第 1.5 道同逻辑 + 同样的围栏跟踪）
VERIF_SECTION=$(echo "$NOTES" | awk '
  /Verification 结果|verification 结果/ { in_verif=1; next }
  in_verif && /^[*-] \*\*(剩余风险|下游)/ { in_verif=0 }
  in_verif && /^[[:space:]]*```/ { in_fence = !in_fence; next }
  in_verif && !in_fence { print }
')
NOTES_KEYS=$(echo "$VERIF_SECTION" \
  | grep -oE "^[[:space:]]+-[[:space:]]+(install|lint|typecheck|unit-test|integration|e2e|build|coverage|runtime-smoke|manual):" \
  | sed -E 's/^[[:space:]]+-[[:space:]]+([a-z-]+):.*$/\1/' \
  | sort -u)
# guard：本段依赖 SOP 步 4 已赋值 VERIFY_KEYS（同脚本上下文）。若单独跑本段致其空 →
# 下方 for 空转 → 集合检查静默失效（与 team Gate 4 第 1.5 道同款兜底，solo 降级为 warn）。
[ -z "$VERIFY_KEYS" ] && echo "⚠️  VERIFY_KEYS 未赋值 — 请先跑 SOP 步 4（s2v_extract_verify_keys），否则本检查无效"
KEY_MISSING=""
for k in $VERIFY_KEYS; do
  echo "$NOTES_KEYS" | grep -qx "$k" || KEY_MISSING="$KEY_MISSING $k"
done
if [ -n "$KEY_MISSING" ]; then
  echo "⚠️  §10『§9 Verification 结果』段缺少 §9 列出的 key（升档到 team 后 Gate 4 第 1.5 道会 BLOCK）：$KEY_MISSING"
fi

# 6. 推进 task spec Status: Ready / In Progress → Done（与 §10 回填合并 commit）
#    Spec Status 状态机见 docs/s2v/standard.md §10.5.1。
#    没有 /s2v-implement skill 的实施 agent（外部 agent / 手动实施）
#    必须在 §10 回填后主动把顶部 **Status** 字段改为 Done — 否则 task 状态机停在
#    Ready，team 升档后主 agent / CI 会按"未完成"处理。
# C8：本 SOP 步 4 的 `s2v_verify_full ... || exit 1` 已隐式绑定 Done（§9 红则脚本
#     在步 4 就退出，到不了这里）—— canonical 路径**无需在此二次复跑**（C8 复审回归：
#     双跑 + §9 含 manual 时重复 /dev/tty 确认 / 非交互 rc2）。
#     ⚠️ 仅当**未照本 SOP 跑**（外部/手动 agent 直接改 Status）时，必须先显式：
#         s2v_require_green "$TASK_SPEC" || exit 1     # 复跑 §9（自动排除 manual）
# 推进 Spec Status: Ready / In Progress → Done（portable perl，BSD/macOS + GNU/Linux 通用）
# ⚠️ 不要用 `sed -i ''`（BSD-only，外部 agent 在 Linux CI 上会失败）
perl -i -pe 's/^\*\*Status\*\*: (Ready|In Progress)$/\*\*Status\*\*: Done/' "$TASK_SPEC"
grep -qE "^\*\*Status\*\*: Done$" "$TASK_SPEC" \
  || { echo "🛑 Status 推进失败 — 检查 $TASK_SPEC 顶部"; exit 1; }

git add "docs/specs/tasks/task-<X.Y>-"*.md   # 双引号让 <X.Y> 不被 bash 当输入重定向
git commit -m "docs(spec): 回填 task-<X.Y> §10 Completion Notes + Status → Done"

# 7. Phase 兜底（C1）：solo 档此前**完全没有**这层（team 靠 §4 Gate 3，solo 裸奔）
#    —— 三轮黑盒/dogfood 互证最强缺口（C1）。
#    若本 task 是其所属 phase 的最后一个 task（该 phase 其余 task 均已 Done — agent
#    据 adapter §Phase 状态索引 / task 计数判定），该 phase 收尾必须过 phase 门禁 +
#    跑 §6 端到端 smoke。非最后 task：LAST_TASK_IN_PHASE 保持 0，跳过本步。
LAST_TASK_IN_PHASE=0   # agent：若本 task 是该 phase 最后一个（其余 task 均 Done）→ 改 1
PHASE_SPEC="docs/specs/phases/phase-<N>-<name>.md"   # agent 替换为本 task 所属 phase spec
if [ "$LAST_TASK_IN_PHASE" = "1" ]; then
  s2v_preflight_phase "$PHASE_SPEC" || {
    echo "🛑 STOP: phase §6（阶段级 AC + 端到端 smoke）未填实或 Status 非法 —"
    echo "   该 phase 最后 task 完工前必须把 phase spec §6 填实（集成兜底，不可跳过）。"
    exit 1
  }
  # 按 PHASE_SPEC §6 列出的端到端 smoke 命令执行；全过才算 phase 完成，失败走卡住协议
  # 通过后把 phase spec 顶部 Status 推进到 Done（与 task 同 §10.5.1 状态机）
  perl -i -pe 's/^\*\*Status\*\*: (Draft|Ready|In Progress)$/\*\*Status\*\*: Done/' "$PHASE_SPEC"
  git add "$PHASE_SPEC"
  git commit -m "docs(spec): phase 收尾 — §6 端到端 smoke 通过 (Status → Done)"
fi
```

---

## R3 · 每次 commit 后立即验证 `[branch]`

solo 档虽然直接在 main 上工作，但 agent 误开 chore/feat 分支 / 误 cherry-pick / 误 rebase 仍会发生。每次 commit 必校验：

```bash
EXPECTED=$(git branch --show-current)   # solo 档通常是 main
git commit -m "..." | tee /tmp/c.txt
grep -qE "^\[${EXPECTED} " /tmp/c.txt || {
  echo "BRANCH MISMATCH: 期望 [$EXPECTED] 实际 $(grep -oE '^\[[^ ]+' /tmp/c.txt)"
  exit 1
}
```

---

## R5 · agent 不得自创 task spec

实施前必须在 `<task-spec-pattern>` 中找到对应 task spec。

- **找到** → 严格按 spec §6 AC / §5 Behavior Contract / §7 追踪表 / §9 Verification Plan 执行
- **找不到** → 立刻停下，让用户跑 `/s2v-add task <name>` 生成。**禁止自创 task spec**（违反 SDD 单一事实源）

> solo 档诱惑很大："反正就我一个人，写个 spec 浪费时间，我直接实现吧"——这是 S2V 在 solo 档最容易破口。请抵抗。

---

## §2.5 Commit 节律（仍按完整规范）

每个 task 至少：

| 阶段 | type | 示例 |
|---|---|---|
| RED | `test` | `test(parser): 加 SCEN-2.1.1 ~ 2.1.5 的 5 个 RED 测试` |
| GREEN | `feat` | `feat(parser): 实现 extractHeadings 通过全部测试` |
| REFACTOR（如有） | `refactor` | `refactor(parser): 提取 walkTokens helper` |
| §10 回填 | `docs` | `docs(spec): 回填 task-2.1 §10 Completion Notes` |

Scope 取值统一为模块名（如 `parser` / `cli` / `auth` / `spec` / `agents` / `adapter` / `adr`），避免随意。

---

## git 协作（solo 档简化）

| 操作 | 是否允许 |
|---|---|
| 在 main 直接 `git commit` | ✅ |
| 在 main 直接 `git push`（如有 remote） | ✅ |
| `git rebase` / `cherry-pick` 在 main 上 | ✅（仅未 push 的本地 commit）|
| `git push --force` / `--force-with-lease` 到 main | 🚫 **禁止默认** — 见下方 |
| `git reset --hard` 到任何已 commit 的分支 | 🚫 **禁止默认** — 用 `git revert` 或 `git branch -f` 替代 |
| 开 feature branch | 可选（如想 review 自己的改动） |
| 开 worktree | 不需要 |

### 历史破坏操作的"禁止默认 + 用户明确确认"流程

`git push --force*` 和 `git reset --hard` 即使 solo 档也**不得 agent 自动跑**。如果**真的**需要（例如错 commit 了 secrets），按以下流程：

1. 写 `BLOCKED-history-rewrite.md` 描述：
   - **要做什么**（如 "把 commit `abc123` 从历史移除"）
   - **为什么必须改写历史**（普通 revert 为何不够）
   - **影响范围**（哪些 ref / tag / 别人的 clone 会失效）
   - **回滚方案**（备份 tag 名 / 远程 backup branch 名）
2. 立刻 push 备份：`git push origin HEAD:backup/pre-rewrite-$(date +%s)`
3. commit 上面这份 BLOCKED 文件
4. **等用户读完文件后明确回复"按方案 X 执行"**才能动手
5. 即使是 solo，也建议先思考一下"未来的我看到 git log 突然少了一截会困惑吗"

---

## 卡住协议（solo 档的 BLOCKED 模板）

任一 AC 满足以下**全部**条件即视为"卡住"，必须停手写 BLOCKED 文件，不可硬猜：

1. 同一 AC 连续失败 ≥3 次
2. 已尝试 systematic-debugging 4 阶段（根因 → 模式 → 假设 → 实施）
3. 已检查上游 task spec / ADR 是否有遗漏的契约信息

写 `BLOCKED-task-<X.Y>.md` 然后 commit：

```markdown
# BLOCKED — task-<X.Y>

## 卡住的 AC
- AC<N>: <原文>

## 已尝试方案
1. <尝试 1> → 失败原因
2. <尝试 2> → 失败原因
3. <尝试 3> → 失败原因

## 当前假设
- 我认为根因可能是 X，证据是 Y

## 决策需求（写给"未来的自己" / 求助）
- 选项 A: 修改 spec（具体改哪行）
- 选项 B: 临时绕过（用什么手段）
- 选项 C: Waive 该 AC（按 s2v §12.3 五项填写）

## 当前测试 / 代码状态
- 红测试在 <test-file>:<line>
- 实施代码在 <src-file>:<line>
```

```bash
git add BLOCKED-task-<X.Y>.md
git commit -m "blocked(<scope>): task-<X.Y> AC<N> 求助"
```

> solo 档下 BLOCKED 文件的最大价值是**强制冷静**：写完 5 段后通常自己就能看到下一步该做什么。

---

## 升级到 team

如项目变复杂（加入第 2 个开发者 / agent / 引入 CI / 准备公开发布），跑：

```
/s2v-tier team
```

命令会重生本 AGENTS.md（按 team 模板，含 worktree + PR + 主 agent gate + 完整 BLOCKED 协议）+ 给"升档影响清单"。

升档不影响 main 上历史 commit（按 R6.2 baseline 化）。

---

## 参考（项目内自包含）

- S2V 完整规范：`docs/s2v/standard.md`（项目快照，由 `/s2v-init` 时复制）
- Tier 详细差异：`docs/s2v/standard.md` §4.5
- Tier 决策树：`docs/s2v/tier-decision-tree.md`
- 模板归档：`docs/s2v/templates-used/`（init 时实际使用的 adapter/agents 模板快照）

> 这些文件是 `/s2v-init` 时从全局 skill（步 0 `_s2v_skill_dir` resolver 解析路径；Claude Code 默认 `~/.claude/skills/s2v/`，其他 agent 见 `docs/s2v/standard.md` §22）复制的快照，让协作者 / CI / 外部 agent 即使没安装全局 skill 也能读到完整规范。
