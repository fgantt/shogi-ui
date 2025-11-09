# Material Evaluation Monitoring Plan

This document tracks the observability and follow-up workflow for Task 11.0 once the material evaluation changes ship to production.

## 1. Dashboards

Create a dashboard panel group titled **Material Evaluation** with the following visualizations:

1. **Preset Usage Split**
   - Metric: `material.preset_usage.{research,classic,custom}` counters from evaluation telemetry.
   - Display: stacked area chart (per hour).
   - Alert: warn if `custom` exceeds 5% without a corresponding rollout ticket.

2. **Hand Balance Delta**
   - Metric: `material.hand_balance.mg` / `eg` average per game.
   - Display: dual-line chart with moving average (15 minute window).
   - Alert: warn on |delta| > 1500 for more than 10 minutes (may indicate runaway drops).

3. **Evaluation Throughput**
   - Metric: `material.evaluations_per_second` derived from statistics report.
   - Display: single stat + sparkline.
   - Alert: fail if throughput regresses >10% vs seven-day baseline.

4. **Phase-Weighted Score Distribution**
   - Metric: histogram from `phase_weighted_total` (per completed game).
   - Display: histogram / heatmap for quick drift detection.

## 2. Alert Rules

| Name | Condition | Action |
|------|-----------|--------|
| `material-hand-imbalance` | |hand_balance.mg| > 1500 for 10 min | Page on-call, attach last 5 telemetry snapshots |
| `material-throughput-drop` | evaluations/sec < baseline * 0.9 | Create P1 incident, trigger A/B rollback procedure |
| `material-custom-usage-spike` | `preset_usage.custom` > 5% | Open follow-up ticket to confirm staging rollout |

## 3. Regression Watch

- **Weekly A/B Self-Play**
  - Compare research vs classic tables over 5k games (time controls: 30s + 5s).
  - Capture Elo diff with 95% confidence interval.
  - Log results in `reports/material/weekly-ab.md`.

- **Latency Benchmark**
  - Nightly `cargo bench --bench material_evaluation_performance_benchmarks --features "legacy-tests,material_fast_loop"`.
  - Publish the fast loop vs legacy median times; fail nightly job if regression >5%.

- **Parity Check**
  - Nightly `cargo test --features material_fast_loop material_delta` to ensure optimized path equivalence.

## 4. Incident Response

1. Gather latest telemetry snapshot (`EvaluationStatistics::telemetry().export_json`).
2. Capture affected positions or value-set IDs.
3. File incident report under `incidents/material/YYYY-MM-DD.md` with:
   - Summary
   - Scope
   - Mitigation / Rollback steps
   - Follow-up tasks

## 5. Backlog Grooming

After each release:

- Review deferred stretch goals (incremental evaluation, drop-aware heuristics) and track in Task 12.x roadmap.
- Consolidate tuning team requests from the monitoring dashboard into Jira tickets with labels `material-eval` and `post-release`.
- Archive weekly A/B and benchmark reports; summarize trends during sprint reviews.

## 6. Ownership

- **Primary:** Evaluation Team (on-call rotation).
- **Supporting:** Tuning Team (value-set experiments), Ops Team (dashboard maintenance).

Document revisions should accompany Task 8.x updates in `tasks-task-11.0-material-evaluation-review.md`.
