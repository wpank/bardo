# golem-engagement

`golem-engagement` tracks golem achievements and maintains the graveyard — a persistent record of every golem that has ever lived and died.

## Features

- Achievement system: award milestones based on trading performance, longevity, and clade contributions
- Graveyard: store death records including final USDC balance, lifespan, cause of death, and the golem's visual identity from `golem-creature`
- Leaderboard: rank golems across the clade by longevity, profit, or knowledge contributed at death
- Emit achievement events to `golem-surfaces` for display in terminal and Telegram

## Architecture

`golem-engagement` is in Layer 6 (Surfaces). It subscribes to `GolemEvent` values from the event fabric and the Thanatopsis protocol's death events from `golem-mortality`. Achievement triggers are evaluated asynchronously after each tick; they do not block the heartbeat.

The graveyard is a SQLite table that persists across golem generations. A successor golem can read its predecessor's death record during initialization, giving it context about how its ancestor died.
