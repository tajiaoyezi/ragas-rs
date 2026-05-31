#!/usr/bin/env bash
# scripts/install.sh — 把 S2V skill 安装到指定目录（full-standard.md §22.4 方案 A/B 的可选便捷件）
#
# 用法：
#   bash scripts/install.sh <target-dir>          # 模式一：装 skill 本体（§22.4 A/B）
#   bash scripts/install.sh                         # 同上，交互询问目标目录
#   bash scripts/install.sh --commands <agent> [dest]
#                                                   # 模式二：装某 agent 原生 /s2v-* 命令桩
#                                                   # agent ∈ claude|codex|cursor|opencode|aider
#   bash scripts/install.sh --upgrade <target-dir>  # 模式三：升级已装 skill（备份 → 拷贝 → self-test）
#
# 模式一行为：忠实拷贝 skill（仅排除 .git VCS 元数据）→ 校验 full-standard.md →
#       跑 helper self-test（非阻断，仅告警）→ 打印 S2V_SKILL_DIR 后续指引。
# 模式二行为：把 commands/<agent>/ 薄委托桩拷到该 agent 命令目录（桩运行时自行
#       按 §22 解析 skill；本脚本不内置 §22.2 路径表 —— 同"瘦"原则）。
# 模式三行为：解析目标已装目录 SKILL.md frontmatter 的 version 字段、与源 version 对比 →
#       同版本免做 → 不同版本备份 <target>.bak.<TS> 后拷贝 → 跑 self-test 验证。
#       兜底：目标无 version 字段（<1.1.0 老安装）视为"未知版本"，照升不误。
#
# 设计约束（重要）：**不内置 / 不探测 §22.2 各 agent 默认路径表**。
#   路径发现由运行时 resolver 负责（init.md 步 0 / tier.md 步 0，§22.5 / §22.6）；
#   本脚本只做"拷到用户指定目录 + 校验"，刻意不复制 resolver 的路径列表 ——
#   避免成为 §22.6 两处 inline resolver 之外的第 3 个路径同步点。
#
# 依赖：bash 3.2+、cp / mkdir / basename（POSIX）。Windows 用 git-bash / WSL。
# 退出码：0=成功；1=拷贝 / 校验失败；2=用法 / 参数错。

set -o pipefail

SKILL_SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 源校验：防止从坏副本安装
if [ ! -f "${SKILL_SRC}/full-standard.md" ] || [ ! -f "${SKILL_SRC}/SKILL.md" ]; then
  echo "❌ 源不是有效 S2V skill（缺 full-standard.md / SKILL.md）：${SKILL_SRC}" >&2
  exit 2
fi

# ── 模式二：安装各 agent 原生 slash 命令适配桩（第 2 层）─────────────────
if [ "${1:-}" = "--commands" ]; then
  AGENT="${2:-}"
  if [ -z "$AGENT" ]; then
    printf '%s' "目标 agent (claude|codex|cursor|opencode|aider): "
    read -r AGENT < /dev/tty 2>/dev/null || AGENT=""
  fi
  case "$AGENT" in
    aider)
      cat <<'EOF'
ℹ️  Aider 无自定义 slash 命令系统 —— 无需安装桩。
改用第 1 层（普适派发，已内建于 SKILL.md / 项目 AGENTS.md）：
  - 已初始化项目： aider --read AGENTS.md   （AGENTS.md 含「S2V 命令派发」节）
  - 或直接：       aider --read <skill>/init.md 等 + 自然语言要求执行
EOF
      exit 0
      ;;
    claude)   CMD_SRC="${SKILL_SRC}/commands/claude"   ; CMD_DEST="${3:-${HOME}/.claude/commands}"          ; CMD_KIND=files ;;
    cursor)   CMD_SRC="${SKILL_SRC}/commands/cursor"   ; CMD_DEST="${3:-${HOME}/.cursor/commands}"          ; CMD_KIND=files ;;
    opencode) CMD_SRC="${SKILL_SRC}/commands/opencode" ; CMD_DEST="${3:-${HOME}/.config/opencode/commands}" ; CMD_KIND=files ;;
    codex)    CMD_SRC="${SKILL_SRC}/commands/codex"    ; CMD_DEST="${3:-${HOME}/.agents/skills}"            ; CMD_KIND=dirs  ;;
    *)
      echo "❌ 未知 agent: '${AGENT}'（支持 claude|codex|cursor|opencode|aider）" >&2
      exit 2
      ;;
  esac
  [ -d "$CMD_SRC" ] || { echo "❌ 缺少源桩目录：$CMD_SRC" >&2; exit 1; }
  mkdir -p "$CMD_DEST" 2>/dev/null || { echo "❌ 无法创建目标：$CMD_DEST" >&2; exit 1; }
  CMD_DEST_ABS="$(cd "$CMD_DEST" && pwd)" || { echo "❌ 无法解析目标：$CMD_DEST" >&2; exit 1; }
  echo "▶ 安装 ${AGENT} slash 命令桩: ${CMD_SRC} -> ${CMD_DEST_ABS}"
  cmd_n=0
  if [ "$CMD_KIND" = files ]; then
    for f in "$CMD_SRC"/s2v-*.md; do
      [ -e "$f" ] || continue
      cp "$f" "${CMD_DEST_ABS}/$(basename "$f")" || { echo "❌ 拷贝失败：$f" >&2; exit 1; }
      echo "  + $(basename "$f")"; cmd_n=$((cmd_n + 1))
    done
  else
    for d in "$CMD_SRC"/s2v-*; do
      [ -d "$d" ] || continue
      b="$(basename "$d")"
      rm -rf "${CMD_DEST_ABS}/${b}"
      cp -R "$d" "${CMD_DEST_ABS}/${b}" || { echo "❌ 拷贝失败：$d" >&2; exit 1; }
      echo "  + ${b}/SKILL.md"; cmd_n=$((cmd_n + 1))
    done
  fi
  [ "$cmd_n" -gt 0 ] || { echo "❌ 未在 $CMD_SRC 找到任何 s2v-* 桩" >&2; exit 1; }
  cat <<EOF

✅ 已安装 ${cmd_n} 个 ${AGENT} 命令桩 -> ${CMD_DEST_ABS}
  - 命令：/s2v-prd /s2v-init /s2v-implement /s2v-add /s2v-tier
  - 桩为薄委托：运行时按 §22（\$S2V_SKILL_DIR 或默认路径）解析 skill 目录后执行对应文档
  - 前提：先用 'bash scripts/install.sh <dir>' 装好 skill 本体（或已设 \$S2V_SKILL_DIR）
  - ${AGENT} 重启 / 新会话后 '/' 输入框可见（Claude Code 下拉含 description 提示）
EOF
  exit 0
fi

# 解析 SKILL.md frontmatter 的 version 字段（供 --upgrade 与诊断使用）。
# 行为：在 `---` 标定的 frontmatter 块内匹配 `^version:[ \t]+...`，剥行尾注释 + 去除空白后输出。
# 兜底：找不到则输出空串（caller 视为"未知版本"，<1.1.0 的老安装走此路径）。
_s2v_parse_version() {
  awk '
    /^---[ \t]*$/ { f = !f; if (!f) exit; next }
    f && /^version:[ \t]+/ {
      sub(/^version:[ \t]+/, "")
      sub(/[ \t]*#.*$/, "")
      gsub(/[ \t\r]/, "")
      print
      exit
    }
  ' "$1"
}

# ── 模式三：--upgrade <target> ──────────────────────────────────────────
# 检测目标已装版本、对比、备份、拷贝、跑 self-test。
if [ "${1:-}" = "--upgrade" ]; then
  UPG_TARGET="${2:-}"
  if [ -z "$UPG_TARGET" ]; then
    printf '%s' "已装 S2V skill 目录 (含 SKILL.md / full-standard.md): "
    read -r UPG_TARGET < /dev/tty 2>/dev/null || UPG_TARGET=""
  fi
  [ -n "$UPG_TARGET" ] || { echo "❌ 未提供目标目录。用法：bash scripts/install.sh --upgrade <target-dir>" >&2; exit 2; }
  case "$UPG_TARGET" in
    "~")   UPG_TARGET="$HOME" ;;
    "~/"*) UPG_TARGET="${HOME}/${UPG_TARGET#\~/}" ;;
  esac
  UPG_ABS="$(cd "$UPG_TARGET" 2>/dev/null && pwd)" || { echo "❌ 目录不存在：$UPG_TARGET" >&2; exit 1; }
  if [ ! -f "${UPG_ABS}/SKILL.md" ] || [ ! -f "${UPG_ABS}/full-standard.md" ]; then
    echo "❌ 目标不是有效 S2V skill 安装（缺 SKILL.md 或 full-standard.md）：${UPG_ABS}" >&2
    echo "   首次安装请用：bash scripts/install.sh <target-dir>" >&2
    exit 1
  fi
  if [ "$UPG_ABS" = "$SKILL_SRC" ]; then
    echo "❌ 目标与源是同一目录，无需升级：$UPG_ABS" >&2
    exit 2
  fi

  SRC_VER="$(_s2v_parse_version "${SKILL_SRC}/SKILL.md")"
  DST_VER="$(_s2v_parse_version "${UPG_ABS}/SKILL.md")"
  if [ -z "$SRC_VER" ]; then
    echo "❌ 源 SKILL.md 无 version 字段（仓库异常 — 应在 frontmatter 声明 version: x.y.z）" >&2
    exit 1
  fi
  echo "▶ 升级 S2V skill: ${UPG_ABS}"
  echo "  当前版本：${DST_VER:-未知（<1.1.0 老安装）}"
  echo "  目标版本：${SRC_VER}"

  if [ -n "$DST_VER" ] && [ "$SRC_VER" = "$DST_VER" ]; then
    echo "✅ 已是最新（${SRC_VER}），无需升级"
    exit 0
  fi

  TS="$(date +%Y%m%d-%H%M%S)"
  BAK="${UPG_ABS}.bak.${TS}"
  echo "▶ 备份现有安装 -> ${BAK}"
  cp -R "$UPG_ABS" "$BAK" || { echo "❌ 备份失败：${BAK}" >&2; exit 1; }

  echo "▶ 应用升级 ${DST_VER:-(unknown)} -> ${SRC_VER}"
  for entry in "$SKILL_SRC"/* "$SKILL_SRC"/.[!.]*; do
    [ -e "$entry" ] || continue
    base="$(basename "$entry")"
    [ "$base" = ".git" ] && continue
    rm -rf "${UPG_ABS}/${base}"
    cp -R "$entry" "${UPG_ABS}/${base}" || { echo "❌ 拷贝失败：$base（备份保留于 ${BAK}）" >&2; exit 1; }
  done

  if [ ! -f "${UPG_ABS}/full-standard.md" ]; then
    echo "❌ 升级后校验失败：${UPG_ABS}/full-standard.md 不存在（备份保留于 ${BAK}）" >&2
    exit 1
  fi
  NEW_VER="$(_s2v_parse_version "${UPG_ABS}/SKILL.md")"
  echo "✅ 文件就位（version=${NEW_VER:-未知}）"

  if [ -f "${UPG_ABS}/scripts/lib/_self-test.sh" ]; then
    echo "▶ 跑 helper self-test ..."
    if bash "${UPG_ABS}/scripts/lib/_self-test.sh" > /tmp/s2v-upgrade-selftest.txt 2>&1; then
      tail -n 1 /tmp/s2v-upgrade-selftest.txt
      echo "✅ self-test 通过"
    else
      echo "⚠️  self-test 未全绿（不阻断；详见 /tmp/s2v-upgrade-selftest.txt；如需回滚：rm -rf ${UPG_ABS} && mv ${BAK} ${UPG_ABS}）" >&2
    fi
  fi

  cat <<EOF

✅ 升级完成 -> ${UPG_ABS}
  ${DST_VER:-(unknown)} -> ${SRC_VER}
  备份：${BAK}（确认无碍后可手工删除）
  变更：见仓库 CHANGELOG.md
EOF
  exit 0
fi

# 解析 target：参数优先，缺省则交互询问
TARGET="${1:-}"
if [ -z "$TARGET" ]; then
  printf '%s' "S2V skill 安装目标目录（如 ~/.claude/skills/s2v；各 agent 默认路径见 full-standard.md §22.2）: "
  read -r TARGET < /dev/tty 2>/dev/null || TARGET=""
fi
if [ -z "$TARGET" ]; then
  echo "❌ 未提供 target 目录。用法：bash scripts/install.sh <target-dir>" >&2
  exit 2
fi

# 展开起始 ~（read / 位置参数不自动做 ~ 展开）
case "$TARGET" in
  "~")   TARGET="$HOME" ;;
  "~/"*) TARGET="${HOME}/${TARGET#\~/}" ;;
esac

mkdir -p "$TARGET" 2>/dev/null || { echo "❌ 无法创建目录：$TARGET" >&2; exit 1; }
TARGET_ABS="$(cd "$TARGET" && pwd)" || { echo "❌ 无法解析目标路径：$TARGET" >&2; exit 1; }
if [ "$TARGET_ABS" = "$SKILL_SRC" ]; then
  echo "❌ 目标与源是同一目录，无需安装：$TARGET_ABS" >&2
  exit 2
fi

# 忠实拷贝（逐顶层条目；先删后拷使重装幂等）。仅排除 .git ——
# 嵌套 .git 会污染宿主目录 git 状态且体积无意义；这是任何"部署文件夹"
# 脚本的通用正确性处理，与具体目录无关。
echo "▶ 安装 S2V skill: ${SKILL_SRC} -> ${TARGET_ABS}"
for entry in "$SKILL_SRC"/* "$SKILL_SRC"/.[!.]*; do
  [ -e "$entry" ] || continue
  base="$(basename "$entry")"
  [ "$base" = ".git" ] && continue
  rm -rf "${TARGET_ABS}/${base}"
  cp -R "$entry" "${TARGET_ABS}/${base}" || { echo "❌ 拷贝失败：$base" >&2; exit 1; }
done

# 安装后校验：resolver 判定"有效 skill dir"的唯一判据 = 含 full-standard.md
if [ ! -f "${TARGET_ABS}/full-standard.md" ]; then
  echo "❌ 安装后校验失败：${TARGET_ABS}/full-standard.md 不存在" >&2
  exit 1
fi
echo "✅ 文件就位（${TARGET_ABS}/full-standard.md 校验通过）"

# helper self-test（skill 仓库自带 _self-test.sh；非阻断 — 失败仅告警不回滚）
if [ -f "${TARGET_ABS}/scripts/lib/_self-test.sh" ]; then
  echo "▶ 跑 helper self-test ..."
  if bash "${TARGET_ABS}/scripts/lib/_self-test.sh" > /tmp/s2v-install-selftest.txt 2>&1; then
    tail -n 1 /tmp/s2v-install-selftest.txt
    echo "✅ self-test 通过"
  else
    echo "⚠️  self-test 未全绿（不阻断安装；详见 /tmp/s2v-install-selftest.txt）" >&2
  fi
fi

# 后续指引（§22.4：默认路径自动发现 / 否则方案 B 设环境变量）
cat <<EOF

✅ S2V skill 安装完成 -> ${TARGET_ABS}

后续（详见 ${TARGET_ABS}/full-standard.md §22）：
  - 目标是某 agent 默认路径（如 ~/.claude/skills/s2v，§22.2）-> resolver 自动发现，无需配置
  - 否则（§22.4 方案 B）-> 在 shell 配置中声明：
      export S2V_SKILL_DIR="${TARGET_ABS}"
  - 验证：在任意 S2V 项目根跑 /s2v-init（resolver 会用上述路径）
EOF
