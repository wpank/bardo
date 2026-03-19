# golem-ta

`golem-ta` provides technical analysis for golems: market regime detection, topological data analysis (TDA) of price manifolds, and classical indicator computation.

## Features

- Regime detection: classify the current market as trending, mean-reverting, or noisy using statistical tests
- Topological data analysis: compute persistent homology features from price time series to detect structural pattern changes
- Classical indicators: moving averages, RSI, Bollinger Bands, VWAP, ATR, and others
- Streaming computation: update indicators incrementally as new price data arrives, without recomputing from scratch

## Architecture

`golem-ta` is in Layer 4 (Infrastructure). The heartbeat's analyze step calls into `golem-ta` to characterize current market conditions. The output — regime label, volatility estimate, key support/resistance levels — feeds into the gate step's decision about whether to proceed with execution.

TDA features are computed less frequently than classical indicators. They are useful for detecting regime transitions that classical indicators lag.
