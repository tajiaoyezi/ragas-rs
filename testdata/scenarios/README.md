# ragas-rs 测试场景数据集 (`testdata/scenarios`)

一批为 ragas-rs 设计的**真实使用场景**评测数据集,用于大量手动/自动测试。每个领域做"好/坏对照",且**只改一个变量**来隔离目标指标,方便你直观看到分数被拉开。

## 这些文件是什么

- 每个 `.jsonl` 文件 = 一个评测数据集,每行一个 `single_turn` 样本。
- 字段:`sample_type`(固定 `single_turn`)、`user_input`、`response`、`retrieved_contexts[]`、`reference`(可选)、`metadata{}`(每个样本的 `note` 标注了它的设计意图)。
- 命名按编号分组(01-19),成对的 `*-grounded/hallucinated`、`*-hit/miss`、`*-verbatim/paraphrase` 等是**对照组**。

## CLI `ragas evaluate` 实际计算哪些指标

| 指标 | 何时计算 | 需要的字段 |
|---|---|---|
| `rouge_l` | **永远**(离线,无需 API key) | `reference`(缺了该样本会报错,符合预期) |
| `faithfulness` | 配了 API key 时 | `response` + `retrieved_contexts` |
| `context_recall` | 配了 key **且**有 `reference` 时 | `reference` + `retrieved_contexts` |

> `answer_relevancy` 和 `context_precision` 是**库级**指标,CLI 的 `evaluate` 不计算(见下方 `access=library` 的场景)。

## 怎么跑

```powershell
# 离线(只算 rouge_l,零 API 消耗)—— 验证数据能解析 + 看 ROUGE 字面差异
.\scripts\run-scenarios.ps1 -Offline

# 在线(配好 .env 里的 key 后,额外算 faithfulness / context_recall)
.\scripts\run-scenarios.ps1

# 单个文件手动跑
cargo run --bin ragas -- evaluate --dataset testdata\scenarios\01-it-helpdesk-grounded.jsonl --report out.json
```

看 discrimination 的方法:**比较对照组的均值**。例如 `01-grounded` 的 `faithfulness` 均值应明显高于 `02-hallucinated`;`11-verbatim` 的 `rouge_l` 应明显高于 `12-paraphrase`(即使语义相同——ROUGE 只看字面)。

## 场景清单


### `it-helpdesk` — Enterprise IT helpdesk knowledge base

- **隔离指标轴:** `faithfulness`  |  **路径:** `cli`
- **对照设计:** Controlled variables held byte-identical across both files per row: user_input, retrieved_contexts, and reference (verified programmatically — all 5 rows match exactly). ONLY the response field differs. The grounded responses assert only facts present in their contexts; the hallucinated responses inject invented specifics absent from (or contradicting) the contexts: fake version numbers (Outlook build 16.0.17328, GlobalProtect 6.2.3), made-up commands (olkrepair /resetcreds), wrong SLAs (P2 stated as 15-min/2-hour, which is actually P1), wrong subnets/portals (172.16.0.0/12, vpn-emea, full vs split tunneling), and fabricated approval/admin steps. Because context_recall is derived from reference + retrieved_contexts only (response-independent) and those are identical, context_recall is HIGH and equal in BOTH files — it does not discriminate. rouge_l is computed in both files (CLI always runs it): grounded mean ~0.75 vs hallucinated mean ~0.54 (re-derived via the crate's whitespace-lowercase LCS-recall); it shifts somewhat but is NOT the clean separator since hallucinated responses retain most reference wording. The decisive, isolated axis is faithfulness: grounded mean HIGH (every claim backed by contexts) vs hallucinated mean LOW (each response contains multiple claims absent from the contexts, lowering supported/total).

  - **`01-it-helpdesk-grounded.jsonl`** — grounded (good)
    - 期望: `faithfulness=high`, `context_recall=high`, `rouge_l=high`
    - 看点: faithfulness mean should be HIGH (near 1.0): every response claim — SSPR/Authenticator/12-char policy/15-min sync, Credential Manager steps, P2 30-min/4-hour SLA + Tier2/Tier3, GlobalProtect portal/10.0.0.0-8/split-tunnel, ServiceNow/Intune/two-business-days — is directly stated in that row's retrieved_contexts. context_recall HIGH (reference sentences are all attributable to the contexts). rouge_l mid/high (mean ~0.75) because responses reuse the reference's exact words in order.
  - **`02-it-helpdesk-hallucinated.jsonl`** — hallucinated (bad)
    - 期望: `faithfulness=low`, `context_recall=high`, `rouge_l=mid`
    - 看点: faithfulness mean should be LOW: each response keeps the grounded scaffolding but injects fabricated specifics absent from (or contradicting) the contexts — 16-char/24-history password rules + BitLocker call, fake Outlook build 16.0.17328 + olkrepair command, wrong P2 SLA (15-min/2-hour = actually P1) + Tier1 round-the-clock + fake duty manager, GlobalProtect 6.2.3 + vpn-emea + 172.16.0.0/12 + full tunneling, director sign-off + Developer Mode + local admin. Those unsupported claims drive supported/total down. context_recall stays HIGH (same contexts+reference as grounded); rouge_l drops only moderately (mean ~0.54) since most reference wording is retained — so faithfulness, not rouge_l, is the discriminator.

### `medical` — Consumer medical-information assistant (general health Q&A over a drug/condition leaflet)

- **隔离指标轴:** `faithfulness`  |  **路径:** `cli`
- **对照设计:** Matched pairs across 03-medical-grounded.jsonl and 04-medical-overclaim.jsonl: the controlled variables user_input, retrieved_contexts, and reference are byte-for-byte identical line-for-line across the two files; ONLY the response changes. The grounded response stays within the leaflet contexts (every claim traceable to retrieved_contexts); the overclaim response keeps the grounded sentence verbatim but appends fabricated dosages, efficacy guarantees, or off-label uses that are absent from the contexts. This isolates faithfulness as the single varied axis. Note on rouge_l: because rouge_l is LCS RECALL over the reference's words (lcs / reference_token_count), appending extra unsupported words to the overclaim response does not remove the reference's word-subsequence from the response, so rouge_l does NOT meaningfully discriminate the two files; faithfulness is the only metric that separates them, exactly as intended. context_recall depends only on reference+contexts (both held identical) and is therefore the same HIGH value in both files.

  - **`03-medical-grounded.jsonl`** — grounded (good)
    - 期望: `faithfulness=high`, `rouge_l=high`, `context_recall=high`
    - 看点: Run `ragas evaluate` with an API key on this file: faithfulness mean should be HIGH (near 1.0) because every response claim is backed by retrieved_contexts. rouge_l should be HIGH because responses echo the reference wording. context_recall should be HIGH because each reference sentence is present in the contexts. Compare faithfulness against the paired 04-medical-overclaim.jsonl: this file should score clearly higher.
  - **`04-medical-overclaim.jsonl`** — overclaiming (bad)
    - 期望: `faithfulness=low`, `rouge_l=high`, `context_recall=high`
    - 看点: Run `ragas evaluate` with an API key on this file: faithfulness mean should be LOW (well below the grounded file) because each response appends one or more claims (doubled doses, guarantees, off-label uses) that have no support in retrieved_contexts; supported/total falls. rouge_l should remain HIGH and roughly equal to the grounded file (it is recall over the unchanged reference, so appended words do not reduce it) -- confirming rouge_l does NOT catch the overclaim. context_recall should be HIGH and identical to the grounded file. The faithfulness gap between this file and 03-medical-grounded.jsonl is the headline signal.

### `chinese-kb` — 中文企业知识库问答(公司制度/产品 FAQ,简体中文)

- **隔离指标轴:** `faithfulness`  |  **路径:** `cli`
- **对照设计:** 成对样本(5 对),每对只改 response。控制变量在两个文件间逐行 byte-identical:user_input、retrieved_contexts、reference 完全相同(已用脚本断言通过)。唯一变化的轴是 response 是否忠于 context —— grounded 文件的回答只陈述 context 第X条里出现的事实;hallucinated 文件用同问题/同 context/同 reference,但把 context 里的数字、期限、政策(年假天数、报销期限、试用期、密码重置渠道、加班倍率)替换成 context 没有的编造值。因此 grounded 与 hallucinated 之间的唯一受控差异就是 faithfulness(回答是否被 context 支持)。context_recall 与 response 无关(只看 reference vs contexts),两文件相同,故两文件方向一致(HIGH)。

  - **`05-chinese-kb-grounded.jsonl`** — grounded 有据 (good)
    - 期望: `faithfulness=high`, `rouge_l=mid`, `context_recall=high`
    - 看点: 运行 CLI(带 API key)后,这 5 条的 faithfulness 应明显高(接近 1.0),且应高于 06 文件同序号样本。rouge_l 离线即出,因 response 复用 reference 原词原数字而处于中高区间。context_recall 应高(reference 信息均在 context 内)。中文 UTF-8 应正确解析不乱码,含转义引号的第4条不应破坏 JSON。
  - **`06-chinese-kb-hallucinated.jsonl`** — hallucinated 编造 (bad)
    - 期望: `faithfulness=low`, `rouge_l=low`, `context_recall=high`
    - 看点: 运行 CLI(带 API key)后,这 5 条 faithfulness 应明显低于 05 文件同序号样本(编造的数字/政策不被 context 支持)。判别 PASS 的标志:逐对 faithfulness(05)显著 > faithfulness(06)。context_recall 应仍为高且与 05 持平(证明它无法识别 response 幻觉),从而凸显 faithfulness 才是隔离轴。rouge_l 因编造 token 替换原数字而相对更低。中文及转义引号需正确解析。

### `ecommerce-returns` — E-commerce customer support — returns & refunds policy

- **隔离指标轴:** `context_recall`  |  **路径:** `cli`
- **对照设计:** Matched pair: user_input, response, and reference are held IDENTICAL across the two files row-by-row; the ONLY variable that differs is retrieved_contexts. Hit file (07) contexts contain the exact return-window / refund-timing / packaging / final-sale / exchange facts the reference asserts, so each reference sentence is attributable to context -> context_recall HIGH. Miss file (08) keeps the same Q/response/reference but swaps in off-topic snippets (shipping speed, order tracking, payment/account security, loyalty points, product care/sizing, gift wrap, newsletter) that omit every returns fact -> reference sentences are unattributable -> context_recall LOW (a genuine retrieval miss). Verified: rouge_l recall(response,reference) is byte-for-byte identical between the two files per row (0.964, 0.913, 0.852, 0.710, 0.886), confirming rouge_l is a held control and retrieved_contexts is the sole moving variable. Caveat reported honestly: faithfulness co-moves with context_recall here because the response states the returns facts, so it is supported by the hit contexts and unsupported by the miss contexts — this is intrinsic to varying contexts while holding the response constant, not a second independent axis.

  - **`07-ecommerce-recall-hit.jsonl`** — retrieval hit (good)
    - 期望: `context_recall=high`, `faithfulness=high`, `rouge_l=high`
    - 看点: Run `ragas evaluate` on this file WITH an API key. Expect context_recall mean near 1.0 (every reference sentence attributable to the hit contexts) and faithfulness mean near 1.0. rouge_l mean ~0.86. Compare directly against 08: same rouge_l, but context_recall (and faithfulness) drop sharply, proving retrieved_contexts is the sole driver.
  - **`08-ecommerce-recall-miss.jsonl`** — retrieval miss (bad)
    - 期望: `context_recall=low`, `faithfulness=low`, `rouge_l=high`
    - 看点: Run `ragas evaluate` on this file WITH an API key. Expect context_recall mean near 0.0 (no reference sentence attributable to the off-topic contexts) and faithfulness near 0.0, while rouge_l stays identical to file 07 (~0.86). The clean signal is: holding rouge_l/response/reference fixed, context_recall collapses purely because retrieval missed the returns facts.

### `hr-policy` — Internal HR policy assistant (PTO, parental leave, expense reimbursement, sick leave, remote-work stipend)

- **隔离指标轴:** `context_recall`  |  **路径:** `cli`
- **对照设计:** Controlled variables held byte-identical across the hit/miss pair: user_input, response, and reference (the reference carries the ground-truth policy facts, e.g. HR-204's 20-day accrual + 5-day rollover). ONLY retrieved_contexts differ. In the HIT file the contexts state the exact policy numbers/figures the reference cites; in the MISS file the contexts retrieved adjacent-but-wrong policy sections (bereavement, jury duty, corporate card, FMLA, equipment loan) plus a process-only blurb of the right policy, so the cited facts are absent. This isolates exactly one axis: whether retrieval surfaced the reference facts. Verified against source: the CLI evaluate path (src/cli/mod.rs) runs rouge_l (offline, always) + FaithfulnessMetric + LlmContextRecallMetric (both LLM, with OPENAI_API_KEY); context_recall under the CLI is the LLM attribution judgment described in the brief, not the offline token-overlap heuristic in src/metrics/rag/mod.rs.

  - **`09-hr-policy-recall-hit.jsonl`** — retrieval hit (good)
    - 期望: `context_recall=high`, `faithfulness=high`, `rouge_l=high`
    - 看点: context_recall mean near 1.0 (every reference sentence attributable to the cited HR-XXX context). faithfulness also near 1.0 since the contexts back each response claim. rouge_l high and equal to the miss file (same response+reference). All five samples score, zero errors. If context_recall is not clearly higher here than in the miss file, the retrieval-hit signal failed to register.
  - **`10-hr-policy-recall-miss.jsonl`** — retrieval miss (bad)
    - 期望: `context_recall=low`, `faithfulness=low`, `rouge_l=high`
    - 看点: context_recall mean clearly LOW / near 0 (reference figures absent from the retrieved wrong-section contexts). faithfulness also low because the response's numeric policy claims are not supported by these contexts. rouge_l stays high and equal to the hit file, proving the only thing that moved between the two files is retrieval quality (context_recall and the context-dependent faithfulness). All five samples score, zero errors.

### `devdocs` — Developer API documentation Q&A (REST endpoints, auth, rate limits, pagination, errors, webhooks)

- **隔离指标轴:** `rouge_l`  |  **路径:** `cli`
- **对照设计:** Matched OFFLINE-runnable pair (rouge_l needs no API key). Across the two files, the controlled variables held IDENTICAL per corresponding line are: user_input, retrieved_contexts, and reference. ONLY the response wording differs. File 11 (verbatim) makes each response reproduce the reference's exact words in their original order (the reference is an in-order subsequence of the response), so LCS = |reference| and rouge_l recall = 1.0. File 12 (paraphrase) conveys the SAME meaning as the SAME reference using synonyms and reordered clauses, so the in-order word overlap is small and rouge_l recall drops to ~0.21-0.35. Verified by re-implementing the source algorithm (src/metrics/traditional/mod.rs rouge_l_recall: whitespace-lowercase tokenizer, per-token strip of non-alphanumeric edges, LCS length / reference token count). The residual paraphrase overlap is only unavoidable shared domain tokens (e.g. Authorization, Bearer, 401, HMAC-SHA256, X-Signature, next_cursor) and stopwords. This isolates exactly one axis: ROUGE is LEXICAL (word-order LCS), not semantic — meaning is preserved in both files yet the score collapses for the paraphrase.

  - **`11-devdocs-verbatim.jsonl`** — lexical match (high rouge_l)
    - 期望: `rouge_l=high`, `faithfulness=high`, `context_recall=high`
    - 看点: rouge_l per-sample scores should be 1.0 on all 5 lines (mean ~1.0). Empirically re-computed with the source algorithm: 1.000 on every line (ref_tokens 20-28). With an API key, faithfulness and context_recall should also be high. This is the HIGH end of the verbatim-vs-paraphrase contrast.
  - **`12-devdocs-paraphrase.jsonl`** — paraphrase (low rouge_l, same meaning)
    - 期望: `rouge_l=low`, `faithfulness=high`, `context_recall=high`
    - 看点: rouge_l per-sample scores should be LOW (~0.21-0.35, mean ~0.25), far below file 11's 1.0, even though meaning and reference are identical -- proving ROUGE is lexical not semantic. Empirically re-computed: 0.250, 0.217, 0.214, 0.238, 0.346. Crucially, with an API key faithfulness should stay HIGH (claims still grounded in the same contexts) and context_recall stays HIGH (reference unchanged), so the ONLY metric that drops between file 11 and file 12 is rouge_l.

### `edge-no-reference` — Developer-tools / cloud-infrastructure support knowledge base (PgBouncer, Stripe webhooks, Kubernetes, Redis)

- **隔离指标轴:** `robustness`  |  **路径:** `cli`
- **对照设计:** Single file, no good/bad pair. The controlled variable held identical across all 4 samples is the absence of the reference key: every line deliberately omits "reference" entirely while keeping a grounded response and >=1 non-empty retrieved_contexts. This isolates per-sample error isolation in the CLI run: rouge_l requires a reference (src/metrics/traditional/mod.rs RougeScore::score returns Err RagasError::Parse "rouge score requires a reference" when sample.reference is None), so it errors once per sample (errors>0, run still completes); faithfulness needs only response + retrieved_contexts, so it still runs and is HIGH; context_recall is not added at all because it requires a reference. No metric direction depends on any varying axis other than the missing reference.

  - **`13-edge-no-reference.jsonl`** — no-reference edge
    - 期望: `rouge_l=error`, `faithfulness=high`, `context_recall=n/a`
    - 看点: Run `ragas evaluate` on this file with an API key. Expect: (1) the run COMPLETES rather than aborting; (2) rouge_l reports an ERROR for all 4 samples (errors>0) because no reference is present, and each error is isolated to its own sample with a message mentioning 'reference'; (3) faithfulness is computed and HIGH (~1.0) on all 4 samples since every response claim is grounded in that sample's retrieved_contexts; (4) context_recall is NOT present in the output at all (it needs a reference). Confirms per-sample error isolation: one undefined metric does not crash the run or block the metrics that do not need a reference.

### `edge-stress` — Edge case: parsing/robustness stress (long multi-paragraph contexts, unicode and emoji, embedded quotes/backslashes, refusal answers, many contexts)

- **隔离指标轴:** `robustness`  |  **路径:** `cli`
- **对照设计:** Single-file robustness stress; the invariant held identical across all lines is the JSONL contract (key order, literal sample_type, present metadata.note, reference on every line); only the parsing hazard varies per line.

  - **`14-edge-stress.jsonl`** — robustness stress
    - 期望: `rouge_l=low`, `rouge_l=high`, `faithfulness=high`, `context_recall=high`, `rouge_l=error`
    - 看点: This file is about PARSING ROBUSTNESS, not score direction. Primary success: `ragas evaluate` ingests all 5 lines without a JSONL parse error (from_jsonl_str succeeds, sample_count=5, scored_samples=5, skipped_samples=0). The escaping hazards must survive: embedded \" double-quotes, the C:\\Users\\svc-app\\logs backslash path, embedded single quotes in the SEC-204 contexts, and real UTF-8 (北京 + 🎉). Secondary (offline rouge_l, always runs): the refusal line (sample 0) must score 0.0000 LOW while samples 1-4 score HIGH (measured 1.0000 / 0.8667 / 1.0000 / 0.8750), and rouge_l errors must be 0 because every line carries a reference. With an API key, faithfulness should be HIGH across the non-refusal lines (every claim is in-context) and effectively n/a/HIGH for the refusal; context_recall should be HIGH for samples 1-4 and the one LOW outlier is sample 0 (its retention reference is absent from the log-shipping contexts). VERIFIED against the real crate: all 5 lines parsed via EvaluationDataset::from_jsonl_str and the rouge_l values came from ragas::rouge_l_recall, not from assertion.

### `mixed-regression` — Realistic mixed production eval set spanning several domains (support, docs, KB)

- **隔离指标轴:** `realistic`  |  **路径:** `cli`
- **对照设计:** Single-file daily regression set, not a good/bad pair, so the held-constant variable is the EVALUATION SETUP itself: all 10 samples share the same CLI metric path (rouge_l offline + faithfulness with key + context_recall with key, since every line carries a reference), the same schema/key-order, and metadata.note as the only quality label. The single axis that varies sample-to-sample is intended sample quality (grounded vs hallucinated vs partial), engineered so the batch MEANS land mid-range. Verified against source: CLI rouge_l = rouge_l_recall = LCS(response_tokens, reference_tokens)/reference_token_count (src/cli/mod.rs:251, src/metrics/traditional/mod.rs:149); faithfulness = LLM decompose-and-verify response vs contexts (src/metric.rs:105); context_recall = LLM attributing reference sentences to contexts (src/metric.rs:519). rouge_l recomputed per line with a faithful Python port of whitespace_lowercase_tokens + LCS: L1/L3/L5/L6=1.00, L7=0.63, L9=0.65, L10=0.58, L2=0.49, L4=0.43, L8=0.30 (mean ~0.66, genuine spread). Key nuance exploited: faithfulness scores the RESPONSE while context_recall scores the REFERENCE, so a hallucinated response (L6, L8) can coexist with high context_recall when retrieval was fine and only the generator invented facts.

  - **`15-mixed-regression.jsonl`** — realistic mixed batch (daily CI regression set): 5 grounded, 3 hallucinated, 2 partial across support/docs/KB domains, each labeled in metadata.note
    - 期望: `faithfulness=mid`, `context_recall=mid`, `rouge_l=mid`
    - 看点: Run `ragas evaluate datasets/15-mixed-regression.jsonl` with an API key. This is a daily-CI regression set: the three CLI metrics should each aggregate to MID-RANGE means with a visible spread, not all-pass or all-fail. Expect faithfulness mean ~0.6-0.7 (5 grounded lines near 1.0, hallucinated L6/L7/L8 low, partial L9/L10 ~0.5); context_recall mean higher ~0.85 (only L7 low and L10 mid, since retrieval is mostly good); rouge_l mean ~0.66 with values ranging 0.30 (L8) to 1.00 (L1/L3/L5/L6). Key cross-metric checks a skeptical engineer should verify: (a) L6 is hallucinated yet rouge_l=1.0 and context_recall high -- proves rouge_l/recall do NOT detect the response's invented next-day-handling clause, only faithfulness does; (b) L6 and L8 show low faithfulness with HIGH context_recall, confirming faithfulness scores the response while context_recall scores the reference; (c) L7 is the only line where both faithfulness AND context_recall drop together, because retrieval is off-topic. No line should error (all carry a non-empty reference, so rouge_l never hits its no-reference error path). Per-line metadata.note states each intended quality for spot-checking.

### `relevancy` — Cloud managed-database / DevOps SaaS product support knowledge base (Postgres-style managed service)

- **隔离指标轴:** `answer_relevancy`  |  **路径:** `library`
- **对照设计:** Matched-pair design across the two files. Held IDENTICAL per pair (the controlled variables): user_input, retrieved_contexts, and reference. The ONLY thing that varies is response. File 16 responses directly and specifically answer the user_input (question and answer embeddings align -> high cosine similarity -> high answer_relevancy). File 17 responses, for the SAME questions/contexts/reference, drift to an adjacent-but-different question or are noncommittal evasive filler ("I'm not sure", generic boilerplate), so the answer embedding sits far from the question embedding -> low answer_relevancy. Because only response changes, the contrast isolates answer_relevancy. Note: answer_relevancy is a LIBRARY-ONLY metric; the CLI does not compute it.

  - **`16-relevancy-ontopic.jsonl`** — on-topic (high relevancy)
    - 期望: `answer_relevancy=high`, `rouge_l=high`, `faithfulness=high`, `context_recall=high`
    - 看点: answer_relevancy should score HIGH: each response is a focused, specific answer to its question, so question/answer embeddings are close. As a bonus, rouge_l is also high because responses mirror the reference wording. Confirm the only difference from file 17 is the response text.
  - **`17-relevancy-offtopic.jsonl`** — off-topic/evasive (low relevancy)
    - 期望: `answer_relevancy=low`, `rouge_l=low`, `faithfulness=mid`, `context_recall=high`
    - 看点: answer_relevancy should score LOW: same five questions/contexts/reference as file 16, but responses either answer an adjacent question (the change-port / exceed-limit drift cases) or are noncommittal filler ('I'm not sure', generic advice). The answer embedding lands far from the question embedding. rouge_l also drops because responses no longer mirror the reference. Confirm context_recall stays high in both files (reference vs contexts unchanged), proving answer_relevancy is the isolated axis.

### `precision` — Cloud developer-platform support knowledge base (managed Postgres, object storage, CI/CD, billing, auth)

- **隔离指标轴:** `context_precision`  |  **路径:** `library`
- **对照设计:** Controlled variables held byte-identical across the clean/noisy pair for each of the 5 samples: user_input, response, reference, and the exact text of the single RELEVANT retrieved context. The ONLY thing that changes is retrieved_contexts ordering+noise. Clean (18): the relevant context is element index 0 with 0-1 distractors after it, so the lone relevant context sits at rank 1 -> precision@1 = 1/1 = 1.0 -> context_precision HIGH. Noisy (19): the SAME relevant context string is buried at the last position after 4-5 irrelevant distractors, so its rank r is 5 or 6 -> precision@r = 1/r in [0.167,0.20] -> context_precision LOW. Because RankedRelevance = sum(precision@k over relevant)/(#relevant) and there is exactly ONE relevant context per sample, the score equals 1/rank of that context, isolating ordering/noise as the single axis. Note: rouge_l, faithfulness, and context_recall are NOT the axis here and (by design) stay roughly equal across the pair because response/reference are identical and the relevant evidence is present in BOTH files; only context_precision separates them. This is a library-only metric; the CLI does not compute it.

  - **`18-precision-clean.jsonl`** — clean retrieval (high precision)
    - 期望: `context_precision=high`, `rouge_l=high`, `faithfulness=high`, `context_recall=high`
    - 看点: context_precision should be 1.0 for every sample because the single relevant context is at rank 1. Compare each line to its 19-precision-noisy.jsonl twin: user_input, response, reference, and the relevant-context string are byte-identical; only the context list ordering/noise differs.
  - **`19-precision-noisy.jsonl`** — noisy retrieval (low precision)
    - 期望: `context_precision=low`, `rouge_l=high`, `faithfulness=high`, `context_recall=high`
    - 看点: context_precision should be ~0.2 (rank 5) or ~0.167 (rank 6) for every sample because the single relevant context is buried last. Confirm against the 18-precision-clean.jsonl twin that response, reference, and the relevant-context string are byte-identical, so the only difference driving the score drop is context ordering/noise. rouge_l/faithfulness/context_recall should NOT drop, demonstrating context_precision is the isolated axis.

## 校验状态(诚实说明)

- 全部 19 个文件经一个对抗式校验 agent 复核过 schema 合法性 + 声称的高低方向是否真的符合指标定义。
- 落盘后用真实二进制**离线**跑过(证明能解析、`rouge_l` 能算出来)。
- `faithfulness` / `context_recall` 的高低判别需要真实 LLM,**由你配 key 后联网跑验证**——这里只保证数据按指标语义"设计正确",未替你联网打分。

