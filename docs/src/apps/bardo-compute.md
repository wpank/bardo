# bardo-compute

bardo-compute is Bardo's computation coordination service. It manages off-chain compute tasks that golems offload when a job is too expensive to run inline during a heartbeat tick — heavy simulation, batch backtesting, or large-scale data processing.

## Features

- Accept and queue compute jobs from golem processes
- Return results asynchronously so golems are not blocked during tick execution
- Coordinate work across multiple compute workers
- Report job status and resource usage back to requesting golems

## Architecture

bardo-compute sits alongside the golem fleet. Golems submit jobs over an internal API and poll for results. This lets a golem's 9-step cognitive loop stay time-bounded even when it needs expensive computations — the golem submits the job, continues other work, and picks up results on a subsequent tick.
