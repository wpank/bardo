# x402 Protocol Specification

> **Reader orientation:** This document specifies the x402 micropayment protocol as used by Bardo. x402 (micropayment protocol where agents pay for inference/compute/data via signed USDC transfers, no API keys) is HTTP-native: a `402 Payment Required` response triggers an ERC-3009 signed authorization, which the server validates before fulfilling the request. It belongs to the `shared/` reference layer and is the payment backbone for Bardo Inference (x402-gated LLM gateway), Styx (global knowledge relay and persistence layer) marketplace, and high-value tool execution. See `prd2/shared/glossary.md` for full term definitions.

## Overview

x402 is the HTTP-native payment protocol used by Bardo for per-request inference billing and marketplace transactions. It extends HTTP with payment headers, enabling micropayments without pre-funded accounts.

## HTTP Flow

1. Client sends request to inference gateway
2. Gateway returns `402 Payment Required` with `X-Payment-Required` header containing: amount, asset (USDC), chain, recipient (operator address), expiry
3. Client constructs ERC-3009 `transferWithAuthorization` signature
4. Client retries original request with `X-Payment` header containing the signed authorization
5. Gateway validates signature, submits payment on-chain (or batches), and fulfills the request

## ERC-3009 Authorization

```
TransferWithAuthorization(
    from: user_address,
    to: operator_address,
    value: amount,
    validAfter: 0,
    validBefore: expiry,
    nonce: random_bytes32
)
```

The `nonce` is random (not sequential), enabling concurrent requests without ordering issues.

## Facilitator API

The facilitator is an on-chain contract that batches multiple x402 payments:
- `batchExecute(Payment[] payments)` -- settles multiple authorizations in one transaction
- Operators run facilitator nodes that batch every ~12 seconds (1 block)
- Failed individual payments are retried in the next batch

## Rust Implementation

```toml
[dependencies]
x402-rs = "0.1"
```

```rust
// Client-side payment flow
async fn pay_and_request(
    gateway: &Url,
    request: Request,
    signer: &dyn Signer,
) -> Result<Response> {
    let resp = client.send(request.clone()).await?;
    if resp.status() != 402 { return Ok(resp); }

    let payment_req = resp.header("X-Payment-Required").parse::<PaymentRequest>()?;
    let authorization = signer.sign_erc3009(&payment_req).await?;

    request.header("X-Payment", authorization.encode());
    client.send(request).await
}
```

## Bardo Integration

- **Inference gateway**: Primary x402 consumer. Each inference request that exceeds the free tier triggers the x402 flow.
- **Styx marketplace**: Strategy purchases use x402 for per-download payment.
- **Tool execution**: High-value on-chain operations may require x402 pre-payment for gas sponsorship.

## Payment Recipient

The `payTo` address is always the operator running the service (inference gateway, marketplace node). Operators set their own margins. Users see one bundled price.

## Token Support

- Primary: USDC on Base (low fees, fast finality)
- Fallback: USDC on Arbitrum, Ethereum mainnet
- Future: any ERC-20 with ERC-3009 support
