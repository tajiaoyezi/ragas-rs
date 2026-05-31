#!/usr/bin/env bash
# scripts/lib/_self-test.sh — Helper self-tests
#
# 跑法：
#   bash scripts/lib/_self-test.sh
#
# 不依赖外部测试框架；用 fixture + assert 函数。
# 任一断言失败立即 exit 1，stdout 打印失败上下文。

set -uo pipefail

# 路径解析
SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_ROOT="$(cd "${SELF_DIR}/../.." && pwd)"
FIXTURE_DIR="$(mktemp -d -t s2v-selftest-XXXXXX)"
trap 'rm -rf "$FIXTURE_DIR"' EXIT

PASS=0
FAIL=0

assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    PASS=$((PASS + 1))
    echo "  ✅ $label"
  else
    FAIL=$((FAIL + 1))
    echo "  ❌ $label"
    echo "     expected: $expected"
    echo "     actual:   $actual"
  fi
}

assert_rc() {
  local label="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    PASS=$((PASS + 1))
    echo "  ✅ $label (rc=$actual)"
  else
    FAIL=$((FAIL + 1))
    echo "  ❌ $label"
    echo "     expected rc: $expected"
    echo "     actual rc:   $actual"
  fi
}

# ─────────────────────────────────────────────────────────
# Fixture 1: 假 adapter（覆盖各种字段语义）
# ─────────────────────────────────────────────────────────
cd "$FIXTURE_DIR"
mkdir -p docs

cat > docs/s2v-adapter.md <<'EOF'
# Adapter

## Commands

- **Install**: pnpm install
- **Lint**: N/A: 项目暂无 lint 配置
- **Typecheck**: pnpm tsc --noEmit
- **Unit Test**: pnpm test
- **Integration tests**: <INTEGRATION_TEST_COMMANDS>
- **E2E tests**:
- **Coverage**: pnpm test --coverage
- **Build**: echo "命令含 :冒号 的边界 case"
- **Runtime smoke**: N/A
EOF

export S2V_ADAPTER_PATH="docs/s2v-adapter.md"

# shellcheck source=adapter.sh
source "${SELF_DIR}/adapter.sh"
# shellcheck source=verify.sh
source "${SELF_DIR}/verify.sh"
# shellcheck source=preflight.sh
source "${SELF_DIR}/preflight.sh"

echo "━━━ adapter.sh: s2v_load_cmd ━━━"
assert_eq "Install 字段值" \
  "pnpm install" \
  "$(s2v_load_cmd Install)"
assert_eq "Lint 显式 N/A 保留原值" \
  "N/A: 项目暂无 lint 配置" \
  "$(s2v_load_cmd Lint)"
assert_eq "命令含冒号不被截断" \
  'echo "命令含 :冒号 的边界 case"' \
  "$(s2v_load_cmd Build)"
assert_eq "未替换占位保留 <...>" \
  "<INTEGRATION_TEST_COMMANDS>" \
  "$(s2v_load_cmd "Integration tests")"
assert_eq "字段缺失返回空" \
  "" \
  "$(s2v_load_cmd "Release")"

echo ""
echo "━━━ verify.sh: s2v_run（语义判读）━━━"

# install 字段为真实命令 → 退出码取决于命令本身（pnpm 不一定装；这里只验"会执行"）
# 用 fake env 避免真跑 pnpm；改测 Coverage 字段的 echo 命令
out="$(s2v_run lint 2>&1)"; rc=$?
assert_rc "lint=N/A 跳过" 0 "$rc"
case "$out" in
  *"⏭  跳过 lint"*) PASS=$((PASS + 1)); echo "  ✅ lint skip 提示文本含 ⏭" ;;
  *) FAIL=$((FAIL + 1)); echo "  ❌ lint skip 提示缺失，输出=$out" ;;
esac

out="$(s2v_run integration 2>&1)"; rc=$?
assert_rc "integration=<占位> hard-fail" 2 "$rc"

# unit-test 自动 required；fixture 里值是 'pnpm test'（真命令）
# 我们不真装 pnpm 验证执行，只验"required 模式 + N/A 时会 hard-fail"
# 改 fixture：临时把 Unit Test 改成 N/A
sed -i.bak 's/^- \*\*Unit Test\*\*:.*$/- **Unit Test**: N\/A/' docs/s2v-adapter.md
out="$(s2v_run unit-test 2>&1)"; rc=$?
assert_rc "unit-test 自动 required，N/A hard-fail" 2 "$rc"
mv docs/s2v-adapter.md.bak docs/s2v-adapter.md

# Coverage 字段是 "pnpm test --coverage"，会真跑（pnpm 大概率不存在 → eval 报错 rc=127）
# 这不是 helper 的问题；helper 行为正确（透传 eval 退出码）。skip。

echo ""
echo "━━━ verify.sh: s2v_extract_verify_keys ━━━"

mkdir -p docs/specs/tasks
cat > docs/specs/tasks/task-1.1-fixture.md <<'EOF'
# Task

## 9. Verification Plan

- Install: pnpm install
- Unit Test: pnpm test
- Coverage: ≥ 80%
- 注：本 task 跳过 e2e 因为没有 e2e 框架
- E2E tests: N/A

## 10. Completion Notes
EOF

keys="$(s2v_extract_verify_keys docs/specs/tasks/task-1.1-fixture.md 2>/dev/null)"
assert_eq "§9 keys 按固定执行序排序（不被字母序打乱）" \
  "install unit-test e2e coverage" \
  "$keys"

# §9 缺 unit-test
cat > docs/specs/tasks/task-no-unit.md <<'EOF'
## 9. Verification Plan

- Install: pnpm install
- Lint: pnpm lint

## 10. Completion Notes
EOF

keys="$(s2v_extract_verify_keys docs/specs/tasks/task-no-unit.md 2>/dev/null)"
out="$(s2v_verify_full "$keys" 2>&1)"; rc=$?
assert_rc "s2v_verify_full 缺 unit-test hard-fail" 2 "$rc"

# 空列表
out="$(s2v_verify_full "" 2>&1)"; rc=$?
assert_rc "s2v_verify_full 空列表 hard-fail" 2 "$rc"

echo ""
echo "━━━ preflight.sh: s2v_preflight_input ━━━"

# 路径形态校验（不需要文件存在的负面 case）
out="$(s2v_preflight_input "" 2>&1)"; rc=$?
assert_rc "空参数" 2 "$rc"
out="$(s2v_preflight_input "/abs/path/task.md" 2>&1)"; rc=$?
assert_rc "拒绝绝对路径" 2 "$rc"
out="$(s2v_preflight_input "./docs/specs/tasks/x.md" 2>&1)"; rc=$?
assert_rc "拒绝 ./ 前缀" 2 "$rc"
out="$(s2v_preflight_input "src/foo.md" 2>&1)"; rc=$?
assert_rc "拒绝非 docs/specs/tasks/ 路径" 2 "$rc"

# 路径合法但文件不存在
out="$(s2v_preflight_input "docs/specs/tasks/task-9.9-missing.md" 2>&1)"; rc=$?
assert_rc "文件不存在" 2 "$rc"

# 路径合法且文件存在
s2v_preflight_input "docs/specs/tasks/task-1.1-fixture.md" >/dev/null 2>&1
assert_rc "合法路径 + 存在 → 通过" 0 "$?"

echo ""
echo "━━━ preflight.sh: s2v_preflight_ready ━━━"

# 完整 fixture：Status=Ready，含 §6 AC + §7 追踪表
cat > docs/specs/tasks/task-1.1-good.md <<'EOF'
# Task

**Status**: Ready

## 6. Acceptance Criteria

- [ ] AC1: foo
- [ ] AC2: bar

## 7. Traceability

| AC | SCEN | TEST | Status |
|---|---|---|---|
| AC1 | SCEN-1.1.1 | TEST-1.1.1 | Not Started |

## 9. Verification Plan

- Unit Test: pnpm test

## 10. Completion Notes
EOF

# Status=Ready + 无 TBD + 有 AC + 有 SCEN → rc=0
out="$(s2v_preflight_ready docs/specs/tasks/task-1.1-good.md 2>&1)"; rc=$?
assert_rc "Ready + 完整字段 → 通过" 0 "$rc"
assert_eq "Ready 状态值（调用方自己读，不依赖 helper export）" \
  "Ready" \
  "$(s2v_read_status docs/specs/tasks/task-1.1-good.md)"

# Status=Draft → rc=1（合法，调用方进 §2A）
sed -i.bak 's/^\*\*Status\*\*: Ready$/**Status**: Draft/' docs/specs/tasks/task-1.1-good.md
rm docs/specs/tasks/task-1.1-good.md.bak
out="$(s2v_preflight_ready docs/specs/tasks/task-1.1-good.md 2>&1)"; rc=$?
assert_rc "Draft → 触发 §2A（rc=1）" 1 "$rc"
assert_eq "Draft 状态值（调用方自己读）" \
  "Draft" \
  "$(s2v_read_status docs/specs/tasks/task-1.1-good.md)"

# Status=Ready 但仍有 TBD → rc=2 hard-fail
sed -i.bak 's/^\*\*Status\*\*: Draft$/**Status**: Ready/' docs/specs/tasks/task-1.1-good.md
rm docs/specs/tasks/task-1.1-good.md.bak
echo "- <TBD-by-user> still here" >> docs/specs/tasks/task-1.1-good.md
out="$(s2v_preflight_ready docs/specs/tasks/task-1.1-good.md 2>&1)"; rc=$?
assert_rc "Ready + 残留 TBD → STOP" 2 "$rc"

# Status=Done → rc=2
cat > docs/specs/tasks/task-done.md <<'EOF'
**Status**: Done

## 6. Acceptance Criteria
- [ ] AC1: x
## 7. Traceability
| | SCEN-1.1.1 | | |
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-done.md 2>&1)"; rc=$?
assert_rc "Done → STOP" 2 "$rc"

# Status=In Progress（多词，不能被 awk '{print $1}' 拆）
cat > docs/specs/tasks/task-inprog.md <<'EOF'
**Status**: In Progress

## 6. Acceptance Criteria
- [ ] AC1: x
## 7. Traceability
| | SCEN-1.1.1 | | |
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-inprog.md 2>&1)"; rc=$?
assert_rc "'In Progress' 多词状态识别" 0 "$rc"
assert_eq "In Progress 多词状态值未被截断" \
  "In Progress" \
  "$(s2v_read_status docs/specs/tasks/task-inprog.md)"

# Status=Ready 但 §6 AC 空 → rc=2
cat > docs/specs/tasks/task-no-ac.md <<'EOF'
**Status**: Ready

## 6. Acceptance Criteria

## 7. Traceability
| | SCEN-1.1.1 | | |
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-no-ac.md 2>&1)"; rc=$?
assert_rc "§6 AC 空 → STOP" 2 "$rc"

# Status=Ready 但 §7 无 SCEN/TEST → rc=2
cat > docs/specs/tasks/task-no-scen.md <<'EOF'
**Status**: Ready

## 6. Acceptance Criteria
- [ ] AC1: x

## 7. Traceability
| AC | BDD | TDD | Status |
|---|---|---|---|
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-no-scen.md 2>&1)"; rc=$?
assert_rc "§7 无 SCEN/TEST → STOP" 2 "$rc"

# 未知 Status 值
cat > docs/specs/tasks/task-bogus.md <<'EOF'
**Status**: Pending

## 6. Acceptance Criteria
- [ ] AC1: x
## 7. Traceability
| | SCEN-1.1.1 | | |
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-bogus.md 2>&1)"; rc=$?
assert_rc "未知 Status 值 → STOP" 2 "$rc"

# DEFECT-1 回归：合规 Ready spec 保留标准 §8.3 渲染规则 <!-- --> 注释 /
# 顶部 Draft 提示 blockquote（二者合法含字面 <TBD-by-user>，标准明示"无需删除"）
# → _s2v_real_tbd_hits 须跳过它们，不得把合规 spec 误杀为 rc=2（主链路阻断）
cat > docs/specs/tasks/task-retained-comment.md <<'EOF'
# Task `1.1`: retained-comment

> ⚠️ 标准保留 blockquote：清零所有 `<TBD-by-user>` 占位后方可 Ready

**Status**: Ready

### 5.2 Imports

- bun:test

## 6. Acceptance Criteria

- [ ] AC1: x

## 7. Traceability

| | SCEN-1.1.1 | | |

<!-- 渲染规则（模式 A）：
     - init/add 完整写出 AC，不挂 <TBD-by-user> 前缀
     - 用户 review 通过无需删除本注释
     - 严禁 `- [ ] <TBD-by-user> AC<N>: 内容` 混合写法
-->

## 10. Completion Notes
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-retained-comment.md 2>&1)"; rc=$?
assert_rc "DEFECT-1: Ready 保留渲染规则注释/blockquote 字面 <TBD> → 不误杀" 0 "$rc"

# DEFECT-1 反向：注释/blockquote 之外有真实内容 <TBD-by-user> → 仍须 STOP
# （证明修复不过度抑制，真实漏填照样抓）
cat > docs/specs/tasks/task-real-tbd.md <<'EOF'
# Task `1.2`: real-tbd

> ⚠️ 标准保留 blockquote：清零所有 `<TBD-by-user>` 再 Ready

**Status**: Ready

### 5.2 Imports

- `<TBD-by-user>`

## 6. Acceptance Criteria

- [ ] AC1: x

## 7. Traceability

| | SCEN-1.1.1 | | |

<!-- 渲染规则：不挂 <TBD-by-user> 前缀；review 通过无需删除本注释 -->

## 10. Completion Notes
EOF
out="$(s2v_preflight_ready docs/specs/tasks/task-real-tbd.md 2>&1)"; rc=$?
assert_rc "DEFECT-1: 内容区真实 <TBD>（排除注释/blockquote 后）仍 STOP" 2 "$rc"

echo ""
echo "━━━ preflight.sh: _s2v_strip_retained（DEFECT-P3-C §10 token 门）━━━"

# DEFECT-P3-C 回归：§10 schema 指引 blockquote 合法含字面 <TBD-after-impl>
# （full-standard.md §10 模板复制进生成 spec，标准明示无需删除）。team Gate 4
# 第 2 道 / solo 升档预检对 $NOTES 做 grep -oE "<...>" 前必须先过
# _s2v_strip_retained，否则一个**正确完工**的 §10 也会因该保留 blockquote
# 被误判出占位 → 误 BLOCK。下面复刻 gate 的真实管道。
NOTES_DONE='> **结构层级（避免数字混淆）**：6 项 outline + 第 7 项 Waiver 登记
>
> `/s2v-implement` 在 task done 时按此格式回填；`/s2v-add task` 在 init 时填占位 `<TBD-after-impl>`。

- **完成日期**：2026-05-16
- **改动文件**：
  - src/settlement.java（新增）
- **commit 列表**：
  - abc1234 feat(settlement): min-transfer
- **§9 Verification 结果**：
  - unit-test: 12 passed / 0 failed
- **剩余风险**：无
- **下游 task 影响**：无'
PLACEHOLDERS=$(echo "$NOTES_DONE" | _s2v_strip_retained | grep -oE "<[A-Za-z_][A-Za-z0-9_-]*>" | sort -u || true)
assert_eq "DEFECT-P3-C: 正确完工 §10 保留指引 blockquote 字面 <TBD-after-impl> → 不误判占位" \
  "" \
  "$PLACEHOLDERS"

# 反向：blockquote 之外字段区真有未替换 <source-file-1> → 仍须被抓（证明不过度抑制）
NOTES_INCOMPLETE='> `/s2v-implement` 在 task done 时按此格式回填；`/s2v-add task` 在 init 时填占位 `<TBD-after-impl>`。

- **完成日期**：2026-05-16
- **改动文件**：
  - `<source-file-1>`（新增/修改）
- **commit 列表**：
  - abc1234 feat: x'
PLACEHOLDERS=$(echo "$NOTES_INCOMPLETE" | _s2v_strip_retained | grep -oE "<[A-Za-z_][A-Za-z0-9_-]*>" | sort -u || true)
assert_eq "DEFECT-P3-C 反向: 字段区真实 <source-file-1>（排除 blockquote 后）仍被抓" \
  "<source-file-1>" \
  "$PLACEHOLDERS"

echo ""
echo "━━━ D-DOC-1: 8 类决策类别单一源（防字面漂移回归）━━━"
# full-standard.md §16.1「8 类决策类别（唯一权威）」表 = 唯一事实源；prd.md /
# init.md 的类别枚举必须与之逐字一致。该 bug 类已漂移/补不全两次（DEFECT-1 →
# P3-C → D-DOC-1 需 de71f23 补完），无守卫必再静默漂 —— 本断言即其回归守卫。
BT=$(printf '\140')                       # 字面反引号，避开 bash 命令替换
FS="$SKILL_ROOT/full-standard.md"; PRD="$SKILL_ROOT/prd.md"; INIT="$SKILL_ROOT/init.md"
CANON=$(awk '
  /^#### 8 类决策类别/ { f=1; next }
  f && /^### /         { exit }
  f && /^\| *[0-9]+ *\| *`/ { if (match($0, /`[^`]+`/)) print substr($0, RSTART+1, RLENGTH-2) }
' "$FS" | sort -u)
assert_eq "§16.1 权威表恰 8 个类别字面" "8" "$(printf '%s\n' "$CANON" | grep -c .)"
DDOC1_MISS=""
while IFS= read -r lit; do
  [ -z "$lit" ] && continue
  grep -qF -- "${BT}${lit}${BT}" "$PRD"  || DDOC1_MISS="$DDOC1_MISS prd:${lit}"
  grep -qF -- "${BT}${lit}${BT}" "$INIT" || DDOC1_MISS="$DDOC1_MISS init:${lit}"
done <<< "$CANON"
assert_eq "prd.md/init.md 含 §16.1 全部 8 权威字面（无删改漂移）" "" "$DDOC1_MISS"
DDOC1_STALE=""
for bad in "测试框架" "部署 / 分发" "部署/分发" "协议 / 接口" "协议/接口"; do
  for ff in "$FS" "$PRD" "$INIT"; do
    grep -qF -- "${BT}${bad}${BT}" "$ff" && DDOC1_STALE="$DDOC1_STALE $(basename "$ff"):${bad}"
  done
done
assert_eq "无旧分歧类别字面被当 canonical（反引号包裹 = 已知漂移形）" "" "$DDOC1_STALE"

echo ""
echo "━━━ C3: s2v_baseline_green 冷启动豁免（排除式+安全偏置，不靠退出码）━━━"
# F-002 + C9' 回归守卫：冷启动豁免按"prune 依赖/docs 后无任何非脚手架文件"判
# （框架无关、排除式、安全偏置），不靠 runner 退出码。关键证伪点：有真实文件但
# unit 真失败时**不能**被当冷启动掩盖（C3-4）；未传 areas 走安全侧（C3-5）。
# 注：C9' 后真实测试文件（.py 等非脚手架）→ 非冷启动 → 跑门禁，语义与旧版一致。
BG_DIR="$FIXTURE_DIR/c3bg"
mkdir -p "$BG_DIR/docs" "$BG_DIR/tests"
cat > "$BG_DIR/docs/adapter.md" <<'EOF'
# Adapter
## Commands
- **Install**: true
- **Typecheck**: true
- **Unit Test**: sh -c 'echo ran > "$BG_MARKER"; exit ${BG_UNIT_RC:-0}'
EOF
BG_SAVE_ADAPTER="${S2V_ADAPTER_PATH:-}"
export S2V_ADAPTER_PATH="$BG_DIR/docs/adapter.md"
export BG_MARKER="$BG_DIR/unit-ran.marker"

# 1) 冷启动：tests/ 空 → 跳过 unit-test，rc0，提示含「冷启动」，marker 不生成
rm -f "$BG_MARKER"
out="$(s2v_baseline_green "$BG_DIR/tests" 2>&1)"; rc=$?
assert_rc "C3-1 冷启动: 无测试文件→基线绿且跳过 unit-test" 0 "$rc"
case "$out" in *"冷启动"*) PASS=$((PASS+1)); echo "  ✅ C3-1 提示文本含「冷启动」" ;;
  *) FAIL=$((FAIL+1)); echo "  ❌ C3-1 冷启动提示缺失，输出=$out" ;; esac
assert_eq "C3-1 冷启动: unit-test 未被执行（无 marker）" "no" \
  "$([ -f "$BG_MARKER" ] && echo yes || echo no)"

# 2) 仅 .gitkeep（空脚手架）仍判冷启动
: > "$BG_DIR/tests/.gitkeep"
rm -f "$BG_MARKER"
out="$(s2v_baseline_green "$BG_DIR/tests" 2>&1)"; rc=$?
assert_rc "C3-2 冷启动: 仅 .gitkeep 仍判冷启动 rc0" 0 "$rc"
assert_eq "C3-2 冷启动: .gitkeep 下 unit-test 未执行" "no" \
  "$([ -f "$BG_MARKER" ] && echo yes || echo no)"

# 3) 有真实测试文件 + unit 通过 → 正常基线绿（执行 unit-test）
echo "x" > "$BG_DIR/tests/test_x.py"
rm -f "$BG_MARKER"; export BG_UNIT_RC=0
out="$(s2v_baseline_green "$BG_DIR/tests" 2>&1)"; rc=$?
assert_rc "C3-3 真实项目: 有测试+unit 通过 → rc0" 0 "$rc"
assert_eq "C3-3 真实项目: unit-test 确被执行" "yes" \
  "$([ -f "$BG_MARKER" ] && echo yes || echo no)"

# 4) 证伪点：有真实测试文件 + unit 失败 → 真红 hard-fail（绝不被冷启动掩盖）
rm -f "$BG_MARKER"; export BG_UNIT_RC=1
out="$(s2v_baseline_green "$BG_DIR/tests" 2>&1)"; rc=$?
assert_rc "C3-4 真红不被掩盖: 有测试+unit 失败 → rc1" 1 "$rc"
assert_eq "C3-4 真红不被掩盖: unit-test 确被执行" "yes" \
  "$([ -f "$BG_MARKER" ] && echo yes || echo no)"

# 5) 证伪点：未传 areas（无法证明冷启动）→ 安全侧仍强制 unit-test
rm -f "$BG_MARKER"; export BG_UNIT_RC=1
out="$(s2v_baseline_green "" 2>&1)"; rc=$?
assert_rc "C3-5 安全侧: 未传 areas→不豁免，unit 失败 rc1" 1 "$rc"
assert_eq "C3-5 安全侧: 未传 areas 时 unit-test 仍执行" "yes" \
  "$([ -f "$BG_MARKER" ] && echo yes || echo no)"

# 6) 证伪点（二轮）：字面占位 <UNIT_TEST_AREAS> → 硬错 rc1（绝不当冷启动）
rm -f "$BG_MARKER"; export BG_UNIT_RC=0
out="$(s2v_baseline_green "<UNIT_TEST_AREAS>" 2>&1)"; rc=$?
assert_rc "C3-6 字面 <UNIT_TEST_AREAS> 未替换 → 硬错 rc1（不掩盖）" 1 "$rc"

# 7) 证伪点（二轮 C3）：多路径中任一不存在(typo) + 真红 → 不冷启动、强制 unit-test rc1
mkdir -p "$BG_DIR/exists_empty"
rm -f "$BG_MARKER"; export BG_UNIT_RC=1
out="$(s2v_baseline_green "$BG_DIR/exists_empty $BG_DIR/missing_typo" 2>&1)"; rc=$?
assert_rc "C3-7 多路径含不存在(typo)+unit 红 → 不冷启动 rc1（不掩盖真红）" 1 "$rc"
assert_eq "C3-7 多路径含不存在 → unit-test 确被执行（安全侧）" "yes" \
  "$([ -f "$BG_MARKER" ] && echo yes || echo no)"

unset BG_UNIT_RC BG_MARKER
export S2V_ADAPTER_PATH="$BG_SAVE_ADAPTER"

echo ""
echo "━━━ C1: s2v_preflight_phase（Phase 层兜底门禁；§6 未填实即 STOP）━━━"
# C1 回归守卫：phase §6（阶段级 AC + 端到端 smoke）必须填实才放行最后 task。
# 关键证伪点：§6 仍含 init 渲染的 <TBD-by-user> → 必 STOP（assert 2）；
# §6 已填实但保留模板 <!-- 注释 -->/blockquote 含字面 <TBD> → 不得误 BLOCK
# （assert 4，与 DEFECT-1/P3-C 同根，证明复用单一源剥除而非裸 grep）。
PH_DIR="$FIXTURE_DIR/c1ph"
mkdir -p "$PH_DIR/docs/specs/phases"
cd "$PH_DIR"
  # 1) §6 已填实 + Status 合法 → rc0
  cat > docs/specs/phases/phase-1-good.md <<'EOF'
**Status**: Done
## 5. 依赖关系
无
## 6. 阶段级 AC + 端到端 smoke
- AC: 用户可完成预订全链路
- smoke: `curl -s localhost:8080/health | grep ok`
## 7. 阶段级风险
无
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-good.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-1 §6 已填实+Status 合法 → 放行" 0 "$rc"

  # 2) 证伪点：§6 仍是 init 渲染的 <TBD-by-user> → STOP
  cat > docs/specs/phases/phase-1-tbd.md <<'EOF'
**Status**: Draft
## 6. 阶段级 AC + 端到端 smoke
<TBD-by-user>（agent 给参考）
## 7. 阶段级风险
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-tbd.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-2 §6 仍 <TBD-by-user> → STOP" 2 "$rc"

  # 3) §6 空 → STOP
  cat > docs/specs/phases/phase-1-empty.md <<'EOF'
**Status**: Draft
## 6. 阶段级 AC + 端到端 smoke

## 7. 阶段级风险
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-empty.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-3 §6 空 → STOP" 2 "$rc"

  # 4) 证伪点：§6 已填实但保留模板注释含字面 <TBD> → 不得误 BLOCK（单一源剥除）
  cat > docs/specs/phases/phase-1-retained.md <<'EOF'
**Status**: Done
## 6. 阶段级 AC + 端到端 smoke
<!-- 渲染规则：init 时此处给 <TBD-by-user> 参考，用户填实后无需删本注释 -->
> 提示：smoke 命令写可执行形式
- AC: 真实阶段验收
- smoke: `make e2e`
## 7. 阶段级风险
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-retained.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-4 §6 填实但留模板注释/blockquote 字面 <TBD> → 不误 BLOCK" 0 "$rc"

  # 5) 非法 Status → STOP
  cat > docs/specs/phases/phase-1-badst.md <<'EOF'
**Status**: Almost Done
## 6. 阶段级 AC + 端到端 smoke
- AC: x
## 7. 阶段级风险
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-badst.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-5 phase Status 非法 → STOP" 2 "$rc"

  # 6) 路径不在 docs/specs/phases/ → STOP（防误用 task 路径）
  s2v_preflight_phase "docs/specs/tasks/task-1.1-x.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-6 非 phase 路径 → STOP" 2 "$rc"

  # 7) Status: 非 bold 渲染也能识别（容忍两种格式）
  cat > docs/specs/phases/phase-1-plain.md <<'EOF'
Status: Done
## 6. 阶段级 AC + 端到端 smoke
- AC: y
## 7. 阶段级风险
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-plain.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-7 Status: 非 bold 渲染仍识别 → 放行" 0 "$rc"

  # 8) 证伪点（claim-1 修复）：§6 已填实但含**示例性尖括号语法** → 不得误 BLOCK。
  #    旧宽口径 <[A-Za-z_]...> 会把 <run>/<pipeline-id> 误判为占位 → rc2（假阳性，
  #    DEFECT-1 同类）；收窄为保留 token 后应 rc0。还原旧正则则本断言翻红（非空壳）。
  cat > docs/specs/phases/phase-1-example.md <<'EOF'
**Status**: Done
## 6. 阶段级 AC + 端到端 smoke
- AC: 用户可用 `goflow status <run>` 查询某次 <pipeline-id> 的运行态
- smoke: `goflow run <pipeline> && goflow status <run> | grep -q done`
## 7. 阶段级风险
无
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-example.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-8 §6 含示例尖括号(<run>/<pipeline-id>) → 不误 BLOCK（旧宽正则会 rc2）" 0 "$rc"

  # 8b) 广度：示例尖括号在 fenced code block 内亦不得误 BLOCK
  cat > docs/specs/phases/phase-1-fence.md <<'EOF'
**Status**: Done
## 6. 阶段级 AC + 端到端 smoke
- AC: CLI 契约如下
```
goflow status <run>
goflow cancel <job-id>
```
- smoke: `make e2e`
## 7. 阶段级风险
无
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-fence.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-8b §6 代码块内示例尖括号 → 不误 BLOCK" 0 "$rc"

  # 9) 保留 token 集完整性（防过度收窄）：<TBD-after-impl> 亦属 S2V 保留占位 → 必 STOP
  cat > docs/specs/phases/phase-1-tbdai.md <<'EOF'
**Status**: Draft
## 6. 阶段级 AC + 端到端 smoke
- AC: <TBD-after-impl>
## 7. 阶段级风险
EOF
  s2v_preflight_phase "docs/specs/phases/phase-1-tbdai.md" >/dev/null 2>&1; rc=$?
  assert_rc "C1-9 §6 含保留 token <TBD-after-impl> → STOP（未过度收窄）" 2 "$rc"

echo ""
echo "━━━ C4: s2v_coverage_threshold_guard（声明阈值则命令必须自我强制）━━━"
# C4 回归守卫：声明阈值但 Coverage 命令不自我强制 → STOP（assert 2 = 64%<阈值
# 静默放行场景）；未声明阈值则不干预（assert 3，防过度阻断）。
C4_DIR="$FIXTURE_DIR/c4cov"
mkdir -p "$C4_DIR"
cat > "$C4_DIR/spec-thr.md" <<'EOF'
## 9. Verification Plan
- Coverage: 覆盖率 ≥ 80%
EOF
cat > "$C4_DIR/spec-nothr.md" <<'EOF'
## 9. Verification Plan
- Coverage: 测量即可（不设门槛）
EOF
cat > "$C4_DIR/ad-tok.md" <<'EOF'
## Commands
- **Coverage**: pytest --cov --cov-fail-under=80
EOF
cat > "$C4_DIR/ad-notok.md" <<'EOF'
## Commands
- **Coverage**: pytest --cov
EOF
cat > "$C4_DIR/ad-na.md" <<'EOF'
## Commands
- **Coverage**: N/A: 仅测量不设门槛
EOF
C4_SAVE="${S2V_ADAPTER_PATH:-}"
export S2V_ADAPTER_PATH="$C4_DIR/ad-tok.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-1 声明阈值+命令含 fail-under → 放行" 0 "$rc"
export S2V_ADAPTER_PATH="$C4_DIR/ad-notok.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-2 声明阈值+命令无强制 → STOP（64%<阈值静默放行场景）" 2 "$rc"
export S2V_ADAPTER_PATH="$C4_DIR/ad-notok.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-nothr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-3 未声明阈值 → 不干预（防过度阻断）" 0 "$rc"
export S2V_ADAPTER_PATH="$C4_DIR/ad-na.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-4 声明阈值 + Coverage=N/A = 矛盾 → STOP（复审修正：不再放行）" 2 "$rc"
# C4-5 证伪点（二轮）：自定义比较脚本**无 s2v-cov-enforced 标记** → STOP（旧 WARN→0 静默放行洞）
cat > "$C4_DIR/ad-awk.md" <<'AWKEOF'
## Commands
- **Coverage**: sh -c 'pytest --cov; c=$(coverage report|tail -1|awk "{print \$NF+0}"); [ "$c" -ge 80 ] || exit 1'
AWKEOF
export S2V_ADAPTER_PATH="$C4_DIR/ad-awk.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-5 自定义脚本无 s2v-cov-enforced 标记 → STOP（堵 WARN→0 静默放行）" 2 "$rc"
# C4-6 证伪点：jacoco:report（仅报告非 check）+ 声明阈值 → STOP
cat > "$C4_DIR/ad-jacoco.md" <<'EOF'
## Commands
- **Coverage**: mvn jacoco:report
EOF
export S2V_ADAPTER_PATH="$C4_DIR/ad-jacoco.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-6 jacoco:report（非 check）+ 声明阈值 → STOP" 2 "$rc"
# C4-7：自定义脚本**带显式 s2v-cov-enforced 标记** → 放行（作者担保的确定性逃生口）
cat > "$C4_DIR/ad-marker.md" <<'EOF'
## Commands
- **Coverage**: sh -c 'pytest --cov; check_cov.sh 80'  # s2v-cov-enforced: check_cov.sh 低于阈值 exit 1
EOF
export S2V_ADAPTER_PATH="$C4_DIR/ad-marker.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-7 自定义脚本+显式 s2v-cov-enforced 标记 → 放行（逃生口）" 0 "$rc"
# C4-8 证伪点（二轮）：pytest --cov && echo ok + 声明阈值 → STOP（旧 && 静默放行洞）
cat > "$C4_DIR/ad-andecho.md" <<'EOF'
## Commands
- **Coverage**: pytest --cov && echo ok
EOF
export S2V_ADAPTER_PATH="$C4_DIR/ad-andecho.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-8 'pytest --cov && echo ok' + 声明阈值 → STOP（堵 && 静默放行）" 2 "$rc"
# C4-9 证伪点（二轮）：jacoco:report && echo check → STOP（旧 jacoco[^|]*check 假阳）
cat > "$C4_DIR/ad-jfalse.md" <<'EOF'
## Commands
- **Coverage**: mvn jacoco:report && echo check
EOF
export S2V_ADAPTER_PATH="$C4_DIR/ad-jfalse.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-9 'jacoco:report && echo check' → STOP（堵旧 jacoco 松散假阳）" 2 "$rc"
# C4-10：精确 jacoco:check（POM verify 绑定）+ 声明阈值 → 放行（精确 token 仍有效）
cat > "$C4_DIR/ad-jcheck.md" <<'EOF'
## Commands
- **Coverage**: mvn -B jacoco:check
EOF
export S2V_ADAPTER_PATH="$C4_DIR/ad-jcheck.md"
s2v_coverage_threshold_guard "$C4_DIR/spec-thr.md" >/dev/null 2>&1; rc=$?
assert_rc "C4-10 jacoco:check（精确 enforce token）+ 声明阈值 → 放行" 0 "$rc"
export S2V_ADAPTER_PATH="$C4_SAVE"

echo ""
echo "━━━ C8: s2v_require_green（Status→Done 前必须 §9 真绿，Done 不可自证）━━━"
# C8 回归守卫：§9 红时 require_green 必 rc1（阻止标 Done — 证伪点 C8-2）。
C8_DIR="$FIXTURE_DIR/c8done"
mkdir -p "$C8_DIR"
cat > "$C8_DIR/spec.md" <<'EOF'
## 9. Verification Plan
- Unit Test
## 10. Completion Notes
EOF
cat > "$C8_DIR/ad-green.md" <<'EOF'
## Commands
- **Unit Test**: true
EOF
cat > "$C8_DIR/ad-red.md" <<'EOF'
## Commands
- **Unit Test**: false
EOF
C8_SAVE="${S2V_ADAPTER_PATH:-}"
export S2V_ADAPTER_PATH="$C8_DIR/ad-green.md"
s2v_require_green "$C8_DIR/spec.md" >/dev/null 2>&1; rc=$?
assert_rc "C8-1 §9 全绿 → 放行 Status→Done" 0 "$rc"
export S2V_ADAPTER_PATH="$C8_DIR/ad-red.md"
s2v_require_green "$C8_DIR/spec.md" >/dev/null 2>&1; rc=$?
assert_rc "C8-2 §9 红 → 阻止 Status→Done（Done 不可自证，证伪点）" 1 "$rc"
s2v_require_green "$C8_DIR/nope.md" >/dev/null 2>&1; rc=$?
assert_rc "C8-3 task spec 不存在 → rc2" 2 "$rc"
# C8-4 证伪点：§9 含 Manual → 复跑排除 manual（不读 /dev/tty / 非交互不 rc2）
cat > "$C8_DIR/spec-manual.md" <<'EOF'
## 9. Verification Plan
- Unit Test
- Manual
## 10. Completion Notes
EOF
export S2V_ADAPTER_PATH="$C8_DIR/ad-green.md"
s2v_require_green "$C8_DIR/spec-manual.md" >/dev/null 2>&1 </dev/null; rc=$?
assert_rc "C8-4 §9 含 Manual → 复跑排除 manual（不卡 /dev/tty / 非交互不 rc2）" 0 "$rc"
# C8-5 证伪点（二轮）：§9 仅 Manual（无 unit-test）→ STOP rc2（守 unit-test 强制契约，
#      堵旧 return 0 —— 外部 agent 跳过 step4 时本可无机械复核标 Done）
cat > "$C8_DIR/spec-manualonly.md" <<'EOF'
## 9. Verification Plan
- Manual
## 10. Completion Notes
EOF
s2v_require_green "$C8_DIR/spec-manualonly.md" >/dev/null 2>&1 </dev/null; rc=$?
assert_rc "C8-5 §9 仅 Manual（无 unit-test）→ STOP rc2（不得无机械复核标 Done）" 2 "$rc"
export S2V_ADAPTER_PATH="$C8_SAVE"

echo ""
echo "━━━ C7: s2v_backfill_notes（§10 骨架自动化 + schema 漂移守卫）━━━"
# C7 守卫：§9 结果行与本 task §9 严格 1:1（不多不少）；C7-3 = 字段 schema 漂移守卫
# （helper 6 字段必须全部在 full-standard §8.3 权威字面，否则自测红 — 同 D-DOC-1 思路）。
C7_DIR="$FIXTURE_DIR/c7notes"
mkdir -p "$C7_DIR"
cat > "$C7_DIR/spec.md" <<'EOF'
## 9. Verification Plan
- Lint
- Unit Test
- Coverage
## 10. Completion Notes
EOF
OUT="$(s2v_backfill_notes "$C7_DIR/spec.md")"; rc=$?
assert_rc "C7-1 backfill 渲染成功" 0 "$rc"
case "$OUT" in *"$(date +%F)"*) PASS=$((PASS+1)); echo "  ✅ C7-1b 含正确 YYYY-MM-DD 完成日期" ;;
  *) FAIL=$((FAIL+1)); echo "  ❌ C7-1b 缺正确日期" ;; esac
for k in lint unit-test coverage; do
  case "$OUT" in *"- ${k}:"*) PASS=$((PASS+1)); echo "  ✅ C7-2 §9 key 行: ${k}" ;;
    *) FAIL=$((FAIL+1)); echo "  ❌ C7-2 缺 §9 key 行: ${k}" ;; esac
done
case "$OUT" in
  *"- build:"*|*"- e2e:"*) FAIL=$((FAIL+1)); echo "  ❌ C7-2 渲染了 §9 未列 key（破坏 1:1）" ;;
  *) PASS=$((PASS+1)); echo "  ✅ C7-2 未渲染 §9 外 key（§10↔§9 1:1）" ;;
esac
FS="$SKILL_ROOT/full-standard.md"
# C7-3 强化（复审）：单一源 = full-standard §8.3「本节定义的权威 schema」枚举行
# （doc 自称权威），不再"全文件随便 grep"。校验：helper 含全部 6 个权威字段名
# + helper 恰 6 个粗体字段（增/删/改名任一漂移即红）。
CANON6="$(grep -oE '本节定义的权威 schema）：.*' "$FS" | head -1 | sed -E 's/^.*）：//')"
C7_MISS=""
C7_OLDIFS="$IFS"; IFS='/'
for nm in $CANON6; do
  nm="$(printf '%s' "$nm" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  [ -z "$nm" ] && continue
  case "$OUT" in *"$nm"*) : ;; *) C7_MISS="$C7_MISS $nm" ;; esac
done
IFS="$C7_OLDIFS"
assert_eq "C7-3a helper 含全部 §8.3 权威 schema 字段（单一源=权威枚举行）" "" "$C7_MISS"
NBOLD="$(echo "$OUT" | grep -oE '\*\*[^*]+\*\*' | sort -u | grep -c .)"
assert_eq "C7-3b helper 恰 6 个粗体字段（防字段增减静默漂移）" "6" "$NBOLD"

echo ""
echo "━━━ C9': greenfield 基线绿重设计（A1 排除式安全偏置 / A2 prune / A3 判定先于 install）━━━"
# Codex 第四轮 NO-GO 回归守卫。C9'-1~4 对"§8.2 旧实现"**可证伪**（mutation-proven）：
#   A1 旧正向白名单 → 未列语言(.vue/.hs)真红被静默掩盖；
#   A2 旧 find 不 prune + install 先跑 → vendored 代码污染判定；
#   A3 旧 install 无条件先于冷启动判定 → 无 manifest greenfield 判定前即死。
C9_DIR="$FIXTURE_DIR/c9prime"
mkdir -p "$C9_DIR"
C9_SAVE="${S2V_ADAPTER_PATH:-}"
export C9_TRACE="$C9_DIR/trace.txt"
# 门禁命令各往 trace 记 token（I/T/U）；exit 码按场景设
cat > "$C9_DIR/ad-failall.md" <<'EOF'
## Commands
- **Install**: sh -c 'echo I >> "$C9_TRACE"; exit 1'
- **Typecheck**: sh -c 'echo T >> "$C9_TRACE"; exit 1'
- **Unit Test**: sh -c 'echo U >> "$C9_TRACE"; exit 1'
EOF
cat > "$C9_DIR/ad-unitred.md" <<'EOF'
## Commands
- **Install**: sh -c 'echo I >> "$C9_TRACE"; exit 0'
- **Typecheck**: sh -c 'echo T >> "$C9_TRACE"; exit 0'
- **Unit Test**: sh -c 'echo U >> "$C9_TRACE"; exit 1'
EOF
cat > "$C9_DIR/ad-tcred.md" <<'EOF'
## Commands
- **Install**: sh -c 'echo I >> "$C9_TRACE"; exit 0'
- **Typecheck**: sh -c 'echo T >> "$C9_TRACE"; exit 1'
- **Unit Test**: sh -c 'echo U >> "$C9_TRACE"; exit 0'
EOF
cat > "$C9_DIR/ad-instfail.md" <<'EOF'
## Commands
- **Install**: sh -c 'echo I >> "$C9_TRACE"; exit 1'
- **Typecheck**: sh -c 'echo T >> "$C9_TRACE"; exit 0'
- **Unit Test**: sh -c 'echo U >> "$C9_TRACE"; exit 0'
EOF

# C9'-1（A1 核心证伪）：未列语言 src/App.vue → 非冷启动 → 跑门禁 → unit 真红 rc1
mkdir -p "$C9_DIR/s1/src"; echo '<template/>' > "$C9_DIR/s1/src/App.vue"
export S2V_ADAPTER_PATH="$C9_DIR/ad-unitred.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s1" && s2v_baseline_green "src" 2>&1)"; rc=$?
assert_rc "C9'-1 未列语言(.vue)源码 → 非冷启动跑门禁 unit 真红 rc1（A1：旧白名单会掩盖→rc0）" 1 "$rc"
assert_eq "C9'-1 unit-test 确被执行（真红 surfaced，未被误判 greenfield 跳过）" "yes" \
  "$(grep -q U "$C9_TRACE" 2>/dev/null && echo yes || echo no)"

# C9'-1b（A1 广度）：另一未列语言 lib/Main.hs → 同样非冷启动跑门禁 rc1
mkdir -p "$C9_DIR/s1b/lib"; echo 'main = pure ()' > "$C9_DIR/s1b/lib/Main.hs"
export S2V_ADAPTER_PATH="$C9_DIR/ad-unitred.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s1b" && s2v_baseline_green "lib" 2>&1)"; rc=$?
assert_rc "C9'-1b 另一未列语言(.hs) → 非冷启动跑门禁 rc1（A1 广度，非个例）" 1 "$rc"

# C9'-2（D-2 仍闭 + 冷启动连 install 一并跳过）：areas=. 仅 go.mod/README/AGENTS/docs（脚手架）
mkdir -p "$C9_DIR/s2/docs/specs"
printf 'module x\n\ngo 1.22\n' > "$C9_DIR/s2/go.mod"
echo '# r' > "$C9_DIR/s2/README.md"; echo 'a' > "$C9_DIR/s2/AGENTS.md"
echo '# s' > "$C9_DIR/s2/docs/specs/p1.md"
export S2V_ADAPTER_PATH="$C9_DIR/ad-failall.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s2" && s2v_baseline_green "." 2>&1)"; rc=$?
assert_rc "C9'-2 greenfield Go areas=.（仅脚手架）→ 冷启动 rc0（D-2 仍闭；旧 §8.2 install 先跑→rc1）" 0 "$rc"
assert_eq "C9'-2 install+typecheck+unit 全未执行（冷启动连 install 一并跳过）" "" \
  "$(cat "$C9_TRACE" 2>/dev/null | tr -d '[:space:]')"

# C9'-3（A2）：areas=. 含预存 node_modules/*.js（vendored）→ prune → 仍 greenfield rc0
mkdir -p "$C9_DIR/s3/node_modules/leftpad"
printf 'module x\n' > "$C9_DIR/s3/go.mod"
echo 'module.exports=1' > "$C9_DIR/s3/node_modules/leftpad/index.js"
export S2V_ADAPTER_PATH="$C9_DIR/ad-failall.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s3" && s2v_baseline_green "." 2>&1)"; rc=$?
assert_rc "C9'-3 node_modules/*.js 被 prune → 仍 greenfield rc0（A2：旧实现 has_code→rc1）" 0 "$rc"
assert_eq "C9'-3 门禁全未执行（vendored 代码未污染判定）" "" \
  "$(cat "$C9_TRACE" 2>/dev/null | tr -d '[:space:]')"

# C9'-4（A3）：install 失败的 greenfield → 判定先于 install → 跳过、不死 rc0
mkdir -p "$C9_DIR/s4/tests"
export S2V_ADAPTER_PATH="$C9_DIR/ad-instfail.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s4" && s2v_baseline_green "tests" 2>&1)"; rc=$?
assert_rc "C9'-4 install 失败的 greenfield → 判定先于 install → rc0 不死（A3：旧 install 先跑→rc1）" 0 "$rc"
assert_eq "C9'-4 install 未被执行（判定在 install 之前）" "no" \
  "$(grep -q I "$C9_TRACE" 2>/dev/null && echo yes || echo no)"

# C9'-5（核心安全回归）：已列语言 src/app.py 真源 + unit 真红 → 非冷启动 rc1，不掩盖
mkdir -p "$C9_DIR/s5/src"; echo 'def f(): pass' > "$C9_DIR/s5/src/app.py"
export S2V_ADAPTER_PATH="$C9_DIR/ad-unitred.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s5" && s2v_baseline_green "src" 2>&1)"; rc=$?
assert_rc "C9'-5 已列语言真源+unit 真红 → 非冷启动 rc1（核心安全：真红不被掩盖）" 1 "$rc"
assert_eq "C9'-5 unit-test 确被执行" "yes" \
  "$(grep -q U "$C9_TRACE" 2>/dev/null && echo yes || echo no)"

# C9'-5b（typecheck 门禁未删）：真源 + typecheck 真红 → 非冷启动 rc1，typecheck 确跑
export S2V_ADAPTER_PATH="$C9_DIR/ad-tcred.md"; : > "$C9_TRACE"
out="$(cd "$C9_DIR/s5" && s2v_baseline_green "src" 2>&1)"; rc=$?
assert_rc "C9'-5b 真源+typecheck 真红 → rc1（typecheck 门禁未被删）" 1 "$rc"
assert_eq "C9'-5b typecheck 确被执行" "yes" \
  "$(grep -q T "$C9_TRACE" 2>/dev/null && echo yes || echo no)"

unset C9_TRACE
export S2V_ADAPTER_PATH="$C9_SAVE"

echo ""
echo "━━━ C9'-MX: 逐语言 greenfield 矩阵（Tier-2：init.md↔verify.sh traceability 守卫）━━━"
# Codex 第四轮 / 外部方案 Tier-2 并入。**单一源 = init.md:273 的 per-language
# UNIT_TEST_AREAS 自荐值**（Go `.` / Go 多目录 `cmd/ internal/` / Python `tests/`
# / TS `test/` / Rust `src/` / Jest `__tests__/`）。逐语言起 greenfield 临时项目，
# 填入 init 自荐 areas + 该语言 manifest，断言 s2v_baseline_green rc0（冷启动跳过）
# —— 这正是此前缺失的引导层↔执行层 traceability，挡"作者没测过的语言"漂移。
# 命令设 fail-if-run 哨兵：rc0 只可能来自正确冷启动跳过（非空壳）。反向：落一个
# 该语言源/测试文件 → 真实内容 → 非冷启动 rc1（证 denylist 永不吞该语言源扩展名）。
MX_DIR="$FIXTURE_DIR/c9mx"; mkdir -p "$MX_DIR"
export MX_TRACE="$MX_DIR/trace.txt"
MX_SAVE="${S2V_ADAPTER_PATH:-}"
while IFS='|' read -r mxlang mxareas mxmani mxsrc; do
  [ -z "$mxlang" ] && continue
  M="$MX_DIR/$mxlang"; mkdir -p "$M/docs/specs"
  echo a > "$M/AGENTS.md"; echo s > "$M/docs/specs/p.md"; : > "$M/$mxmani"
  # init 步 5.5 mkdir 空 areas 目录（多 pathspec 空格分词）
  # shellcheck disable=SC2086
  ( cd "$M" && mkdir -p $mxareas )
  cat > "$M/ad.md" <<EOFMX
## Commands
- **Install**: sh -c 'echo I >> "$MX_TRACE"; exit 1'
- **Typecheck**: sh -c 'echo T >> "$MX_TRACE"; exit 1'
- **Unit Test**: sh -c 'echo U >> "$MX_TRACE"; exit 1'
EOFMX
  export S2V_ADAPTER_PATH="$M/ad.md"
  : > "$MX_TRACE"
  out="$(cd "$M" && s2v_baseline_green "$mxareas" 2>&1)"; rc=$?
  assert_rc "C9'-MX[$mxlang] init 自荐 areas='$mxareas' greenfield → rc0（冷启动跳过·traceability）" 0 "$rc"
  assert_eq "C9'-MX[$mxlang] 门禁全未执行（rc0 非命令意外通过）" "" \
    "$(cat "$MX_TRACE" 2>/dev/null | tr -d '[:space:]')"
  # 反向：落该语言源/测试文件 → 真实内容 → 非冷启动 → install(fail) rc1
  mkdir -p "$M/$(dirname "$mxsrc")"; echo 'x' > "$M/$mxsrc"
  : > "$MX_TRACE"
  out="$(cd "$M" && s2v_baseline_green "$mxareas" 2>&1)"; rc=$?
  assert_rc "C9'-MX[$mxlang] 落 $mxsrc（真实内容）→ 非冷启动 rc1（denylist 未吞该语言源扩展名）" 1 "$rc"
done <<'EOFMXTBL'
go|.|go.mod|main.go
gomulti|cmd/ internal/|go.mod|internal/svc.go
py|tests/|requirements.txt|tests/test_x.py
ts|test/|package.json|test/x.test.ts
rust|src/|Cargo.toml|src/lib.rs
jest|__tests__/|package.json|__tests__/x.test.js
EOFMXTBL
unset MX_TRACE
export S2V_ADAPTER_PATH="$MX_SAVE"

echo ""
echo "━━━ C10: s2v_guard_fixture_tracked（D-3 .log fixture 静默脱 track）━━━"
# 黑盒第四轮 D-3 回归守卫：日志/数据类 fixture 扩展名命中 baseline .gitignore
# （*.log 等），git add 静默拒绝、git status 不显示，而 /s2v-add 仍报 ✅。守卫须
# 在 git add 前探测：命中忽略 → STOP rc1（C10-1）；推荐的 !test/fixtures/ 例外
# 须真生效（C10-2 同时验证给用户的修法不是民间传说）。
C10_DIR="$FIXTURE_DIR/c10fx"
mkdir -p "$C10_DIR/test/fixtures"
( cd "$C10_DIR" && git init -q && git config user.email t@e.x && git config user.name t ) >/dev/null 2>&1
printf '*.log\n*.tmp\n' > "$C10_DIR/.gitignore"
echo "2026-05-17 ERROR boom" > "$C10_DIR/test/fixtures/sample.log"
echo '{"k":1}'              > "$C10_DIR/test/fixtures/sample.json"

# C10-1 证伪点：*.log fixture 命中 .gitignore → STOP rc1（D-3 核心；旧无守卫=静默放行）
( cd "$C10_DIR" && s2v_guard_fixture_tracked "test/fixtures/sample.log" ) >/dev/null 2>&1; rc=$?
assert_rc "C10-1 .log fixture 命中 .gitignore → STOP rc1（堵静默脱 track）" 1 "$rc"

# C10-2 证伪点：加 !test/fixtures/ 例外后**同一 .log** 不再被忽略 → rc0
#   （双重作用：证明 init.md baseline 补的例外语法在 git 里真生效）
printf '*.log\n*.tmp\n!test/fixtures/\n!test/fixtures/**\n' > "$C10_DIR/.gitignore"
( cd "$C10_DIR" && s2v_guard_fixture_tracked "test/fixtures/sample.log" ) >/dev/null 2>&1; rc=$?
assert_rc "C10-2 加 !test/fixtures/ 例外 → 同一 .log 可入库 rc0（修法真实有效）" 0 "$rc"

# C10-3：未命中忽略的常规 fixture（.json）→ rc0（无误报）
( cd "$C10_DIR" && s2v_guard_fixture_tracked "test/fixtures/sample.json" ) >/dev/null 2>&1; rc=$?
assert_rc "C10-3 常规 .json fixture 不被忽略 → rc0（无误报）" 0 "$rc"

# C10-4：缺参数 → rc2；非 git 树 → rc2（与"被忽略 rc1"区分，不混淆环境错）
( cd "$C10_DIR" && s2v_guard_fixture_tracked "" ) >/dev/null 2>&1; rc=$?
assert_rc "C10-4a 缺 fixture 路径参数 → rc2" 2 "$rc"
C10_NOGIT="$FIXTURE_DIR/c10nogit"; mkdir -p "$C10_NOGIT"
( cd "$C10_NOGIT" && s2v_guard_fixture_tracked "x.log" ) >/dev/null 2>&1; rc=$?
assert_rc "C10-4b 不在 git 工作树 → rc2（与 rc1 被忽略区分）" 2 "$rc"

# C10-5 文档守卫：init.md baseline .gitignore 必含 !test/fixtures/ 例外（防 Part B 静默回退）
grep -qF '!test/fixtures/' "$SKILL_ROOT/init.md"; rc=$?
assert_rc "C10-5 init.md baseline .gitignore 含 !test/fixtures/ 例外（防回退）" 0 "$rc"

# C10-6（A8 证伪）：set -u 下**零参**调用 → 必 rc2（旧 `local f=$1` 会 unbound 退出 ≠2）
( s2v_guard_fixture_tracked ) >/dev/null 2>&1; rc=$?
assert_rc "C10-6 set -u 零参 → rc2（A8：旧 local f=\$1 unbound 退出≠2）" 2 "$rc"

# C10-7（A4 证伪）：父目录整体被忽略(/test) → 守卫仍 STOP rc1（安全成立，无静默丢失）
#   且 stderr 文案含"父目录"逐层解忽略指引（A4 修法完整、不误导）
printf '/test\n!test/fixtures/\n!test/fixtures/**\n' > "$C10_DIR/.gitignore"
c10err="$(cd "$C10_DIR" && s2v_guard_fixture_tracked "test/fixtures/sample.log" 2>&1 >/dev/null)"; rc=$?
assert_rc "C10-7 父目录被忽略(/test) → 守卫仍 rc1 STOP（安全成立）" 1 "$rc"
case "$c10err" in
  *"父目录整体被忽略"*) PASS=$((PASS+1)); echo "  ✅ C10-7 文案含父目录逐层解忽略指引（A4 修法完整）" ;;
  *) FAIL=$((FAIL+1)); echo "  ❌ C10-7 文案缺父目录指引（A4 未修/回退）" ;;
esac

echo ""
echo "━━━ C11: s2v_guard_areas_tracked（claim-3a / C10 源码版：声明 SOURCE/UNIT area 被 .gitignore 静默遮蔽）━━━"
# Go/Java dogfood 互证：裸二进制名 `goflow` 同时吞 Go `cmd/goflow/` 源码目录，
# git add 静默跳过 → RED 丢源码/假绿。C10 守卫仅 fixture-scoped，源码目录此前
# 无机械兜底。关键证伪：未track+被忽略 → STOP rc1（C11-1）；根锚定后同一文件
# → rc0（C11-2，证修法真生效）；已 track 即使被忽略 → rc0（C11-4，安全不对称
# 只报真丢失、不对已安全文件 cry-wolf）；干净 area → rc0（C11-3 无误报）；
# 零参/非 git → rc2（C11-5 与 rc1 区分）；父目录整体被忽略 → 仍 rc1（C11-6）。
C11_DIR="$FIXTURE_DIR/c11area"
mkdir -p "$C11_DIR/cmd/goflow" "$C11_DIR/internal"
( cd "$C11_DIR" && git init -q && git config user.email t@e.x && git config user.name t ) >/dev/null 2>&1
printf 'goflow\n' > "$C11_DIR/.gitignore"        # 裸二进制名（无前导 /）→ 同时吞 cmd/goflow/
echo 'package main'     > "$C11_DIR/cmd/goflow/main.go"
echo 'package internal' > "$C11_DIR/internal/svc.go"

# C11-1 证伪点：cmd/goflow/main.go 未 track 且被裸 `goflow` 命中 → STOP rc1
c11err="$(cd "$C11_DIR" && s2v_guard_areas_tracked "." 2>&1 >/dev/null)"; rc=$?
assert_rc "C11-1 area 下未track+被忽略源码(cmd/goflow/main.go) → STOP rc1（堵静默丢源码）" 1 "$rc"
case "$c11err" in
  *"cmd/goflow/main.go"*) PASS=$((PASS+1)); echo "  ✅ C11-1 STOP 文案列出被遮蔽源文件" ;;
  *) FAIL=$((FAIL+1)); echo "  ❌ C11-1 文案未列出被遮蔽文件" ;;
esac

# C11-2 证伪点：根锚定 /goflow（只忽略 repo 根二进制）后同一源码不再被遮蔽 → rc0
printf '/goflow\n' > "$C11_DIR/.gitignore"
( cd "$C11_DIR" && s2v_guard_areas_tracked "." ) >/dev/null 2>&1; rc=$?
assert_rc "C11-2 .gitignore 根锚定 /goflow → cmd/goflow/ 不再被遮蔽 rc0（修法真有效）" 0 "$rc"

# C11-3：干净 area（无遮蔽）→ rc0（无误报）
( cd "$C11_DIR" && s2v_guard_areas_tracked "internal/" ) >/dev/null 2>&1; rc=$?
assert_rc "C11-3 干净 area internal/ → rc0（无误报）" 0 "$rc"

# C11-4 证伪点：文件已 track 即使匹配 .gitignore 也不会丢 → rc0（不对已安全文件 cry-wolf）
printf 'goflow\n' > "$C11_DIR/.gitignore"        # 复原裸名（会匹配 cmd/goflow/main.go）
( cd "$C11_DIR" && git add -f cmd/goflow/main.go && git commit -q -m t ) >/dev/null 2>&1
( cd "$C11_DIR" && s2v_guard_areas_tracked "." ) >/dev/null 2>&1; rc=$?
assert_rc "C11-4 文件已 track（即使匹配忽略）→ rc0（安全不对称：只报真丢失，不误报已安全）" 0 "$rc"

# C11-5：零参 → rc2；非 git 树 → rc2（与"被遮蔽 rc1"区分，不混淆环境错）
( s2v_guard_areas_tracked ) >/dev/null 2>&1; rc=$?
assert_rc "C11-5a 零参 → rc2（set -u 下不 unbound）" 2 "$rc"
C11_NOGIT="$FIXTURE_DIR/c11nogit"; mkdir -p "$C11_NOGIT/src"; echo x > "$C11_NOGIT/src/a.go"
( cd "$C11_NOGIT" && s2v_guard_areas_tracked "src/" ) >/dev/null 2>&1; rc=$?
assert_rc "C11-5b 不在 git 工作树 → rc2（与 rc1 被遮蔽区分）" 2 "$rc"

# C11-6 证伪点（A4 parity）：父目录整体被忽略(/cmd) → 仍 STOP rc1 + 文案含逐层解忽略
printf '/cmd\n' > "$C11_DIR/.gitignore"
echo 'package goflow' > "$C11_DIR/cmd/goflow/util.go"   # 新建未 track 文件复现父目录遮蔽
c11err2="$(cd "$C11_DIR" && s2v_guard_areas_tracked "." 2>&1 >/dev/null)"; rc=$?
assert_rc "C11-6 父目录整体被忽略(/cmd) → 仍 STOP rc1（安全成立，无静默丢失）" 1 "$rc"
case "$c11err2" in
  *"父目录"*) PASS=$((PASS+1)); echo "  ✅ C11-6 文案含父目录逐层解忽略指引" ;;
  *) FAIL=$((FAIL+1)); echo "  ❌ C11-6 文案缺父目录指引" ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "结果：$PASS pass / $FAIL fail"
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
echo "✅ 全部通过"
