# golem-triage

`golem-triage` detects surprising events using Bayesian surprise — the divergence between what the golem expected to observe and what it actually observed. High-surprise events get escalated for deeper analysis; routine observations are batched.

## Features

- Bayesian surprise scoring: compute KL divergence between prior and posterior observation distributions
- Surprise-gated analysis: automatically route high-surprise events to `T1`/`T2` inference instead of `T0`
- Adaptive priors: update the prior distribution after each tick so the golem's expectations stay calibrated
- Anomaly flagging: tag observations that exceed a configurable surprise threshold for inclusion in the current tick's workspace

## Architecture

`golem-triage` is in Layer 4 (Infrastructure). It runs inside the heartbeat's observe step. Each incoming observation is scored against the current prior. High-scoring events are flagged and passed to the analyze step with elevated priority. The prior is updated during the reflect step after the tick completes.

This keeps the golem from spending T2 inference budget on routine observations while still escalating genuinely novel events for deeper reasoning.
