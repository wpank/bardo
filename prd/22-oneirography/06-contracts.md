# On-Chain Contract Architecture [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Networks**: Base (x402 payments, image gen) + Ethereum Mainnet / Base Sepolia (SuperRare Bazaar + Series contracts)
>
> **Depends on**: `golem-custody` (wallet management), `golem-tools` (trust tiers), TypeScript sidecar (Rare Protocol CLI)

---

> **Reader orientation:** This document specifies the on-chain contract architecture for Oneirography NFTs -- how the Golem's (mortal autonomous DeFi agent) Rust runtime interacts with SuperRare's Solidity contracts. It belongs to the **Oneirography creative expression layer** and covers the dual-path integration (TypeScript sidecar for Rare Protocol CLI + Alloy direct calls as fallback), Series contract deployment, x402 payment flow for image generation, Bazaar auction configuration, and network strategy across Base and Ethereum. You should understand ERC-721, smart contract interaction patterns, and cross-chain deployment. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Dual-Path Architecture

The Golem (mortal autonomous DeFi agent) runtime is Rust. The Rare Protocol CLI (`@rareprotocol/rare-cli`) is a Node/TypeScript tool. Two integration paths bridge this gap.

### Path A: TypeScript Sidecar + Rare Protocol CLI (Primary)

The existing TypeScript sidecar (`07-tools/01-architecture.md`) -- the same sidecar that handles Uniswap SDK math that can't be ported to Rust -- is extended with Rare Protocol CLI commands.

Communication uses the existing `SidecarClient` pattern: JSON-RPC 2.0 over a Unix domain socket at `/tmp/golem-sidecar.sock`. Architecturally identical to how the Golem calls `findBestRoute` for Uniswap SDK math. The `SidecarClient::call` method serializes the call as JSON-RPC 2.0, sends over the Unix socket, and awaits the response. The TypeScript server receives these via `socket.on('message', ...)` and dispatches to the appropriate `RareProtocolBridge` method.

### Path B: Alloy Direct Contract Calls (Fallback / Production)

If the CLI doesn't support a needed operation, or for production deployments where shelling out to Node adds latency, the Rust runtime calls SuperRare contracts directly via Alloy.

```rust
// golem-oneirography/src/contracts.rs

use alloy::{
    contract::SolCall,
    primitives::{Address, U256},
    providers::Provider,
};

/// Direct Alloy calls to SuperRare Series + Bazaar contracts.
/// Used as fallback when rare-cli doesn't support an operation,
/// or in production for lower latency.
pub struct SuperRareContracts {
    provider: Arc<dyn Provider>,
    series_address: Address,
    bazaar_address: Address,
    signer: LocalSigner,
}
```

---

## Rare Protocol Series Contracts

Each Golem gets one Series contract. All of its NFTs (dream journals, death masks, mandalas, crucibles, cartographies, self-portraits) are minted on the same Series.

### Deployment

Series contracts are deployed once per Golem, during `on_session_start` if `nft.series_contract` is None and any minting feature is in `effective_features`.

**Sidecar deployment**:

```typescript
// sidecar/src/rare-protocol.ts

export class RareProtocolBridge {
  async deploySeriesContract(params: {
    name: string;        // e.g., "Dreams of AETHER-7b3f"
    symbol: string;      // e.g., "DREAM7B3F"
    ownerAddress: string;
    network: 'ethereum' | 'base' | 'base-sepolia';
  }): Promise<{ contractAddress: string; txHash: string }> {
    const result = execSync(
      `npx @rareprotocol/rare-cli deploy-series \
        --name "${params.name}" \
        --symbol "${params.symbol}" \
        --owner ${params.ownerAddress} \
        --network ${params.network}`,
      { encoding: 'utf-8' }
    );
    return JSON.parse(result);
  }
}
```

**Rust-side call**:

```rust
async fn deploy_series_contract(
    &self,
    ctx: &SidecarClient,
    params: &SeriesParams,
) -> Result<Address> {
    let result = ctx.sidecar.call("rare_deploy_series", serde_json::json!({
        "name": params.name,
        "symbol": params.symbol,
        "owner": params.owner_address.to_string(),
        "network": params.network,
    })).await?;
    Ok(result["contractAddress"].as_str()
        .ok_or_else(|| anyhow!("missing contractAddress"))?.parse()?)
}
```

The returned contract address is persisted to `golem.toml` as `oneirography.nft.series_contract` so subsequent sessions reuse it.

### Minting

**Sidecar mint**:

```typescript
async mintToken(params: {
  seriesContract: string;
  tokenUri: string;  // IPFS URI to metadata JSON
  network: string;
}): Promise<{ tokenId: string; txHash: string }> {
  const result = execSync(
    `npx @rareprotocol/rare-cli mint \
      --contract ${params.seriesContract} \
      --uri ${params.tokenUri} \
      --network ${params.network}`,
    { encoding: 'utf-8' }
  );
  return JSON.parse(result);
}
```

**Rust-side call**:

```rust
async fn mint_dream_nft(&self, ctx: &SidecarClient, token_uri: &str) -> Result<u64> {
    let result = ctx.sidecar.call("rare_mint_token", serde_json::json!({
        "contract": self.series_contract.to_string(),
        "uri": token_uri,
        "network": self.config.network,
    })).await?;
    Ok(result["tokenId"].as_str()
        .ok_or_else(|| anyhow!("missing tokenId"))?.parse()?)
}
```

**Alloy direct mint (fallback)**:

```rust
pub async fn mint(&self, token_uri: &str) -> Result<U256> {
    let call = addNewTokenCall { _uri: token_uri.into() };
    let tx = self.provider
        .send_transaction(call.encode(), self.series_address)
        .await?;
    // Parse token ID from Transfer event
    Ok(parse_token_id_from_receipt(&tx.receipt().await?))
}
```

---

## x402 Payment Flow (StableStudio on Base)

The Golem already manages a wallet on Base (via the custody modes in `01-golem/13-custody.md`). The x402 flow for image generation:

```rust
// golem-oneirography/src/x402.rs

/// x402 payment flow for StableStudio image/video generation.
pub struct StableStudioClient {
    base_provider: Arc<dyn Provider>,  // Base L2 provider
    signer: LocalSigner,               // Base wallet
    usdc_address: Address,              // 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913
}

impl StableStudioClient {
    pub async fn generate_image(&self, prompt: &str, model: &str) -> Result<String> {
        // Step 1: POST without payment -> get 402 + PAYMENT-REQUIRED header
        let resp = reqwest::Client::new()
            .post(format!("https://stablestudio.dev/api/generate/{model}/generate"))
            .json(&serde_json::json!({ "prompt": prompt, "aspectRatio": "1:1", "imageSize": "4K" }))
            .send()
            .await?;

        if resp.status() == 402 {
            let payment_req = decode_payment_required(resp.headers())?;

            // Step 2: Sign USDC authorization on Base
            let signature = self.sign_x402_payment(&payment_req).await?;

            // Step 3: POST with PAYMENT-SIGNATURE header
            let resp = reqwest::Client::new()
                .post(format!("https://stablestudio.dev/api/generate/{model}/generate"))
                .header("PAYMENT-SIGNATURE", signature)
                .json(&serde_json::json!({ "prompt": prompt, "aspectRatio": "1:1", "imageSize": "4K" }))
                .send()
                .await?;

            let job: JobResponse = resp.json().await?;

            // Step 4: Poll until complete
            let image_url = self.poll_job(&job.job_id).await?;
            Ok(image_url)
        } else {
            Err(anyhow!("Expected 402, got {}", resp.status()))
        }
    }
}
```

---

## SuperRare Bazaar Integration

The Bazaar handles all auction mechanics: reserve auctions, scheduled auctions, convertible offers, and bidding.

### Auction Configuration

**Sidecar call**:

```typescript
async configureAuction(params: {
  originContract: string;
  tokenId: string;
  auctionType: 'reserve' | 'scheduled';
  startingAmount: string;   // wei
  currency: string;         // ERC-20 address or 0x0 for ETH
  duration: number;         // seconds
  network: string;
}): Promise<{ txHash: string }> {
  const result = execSync(
    `npx @rareprotocol/rare-cli create-auction \
      --contract ${params.originContract} \
      --token-id ${params.tokenId} \
      --type ${params.auctionType} \
      --starting-amount ${params.startingAmount} \
      --currency ${params.currency} \
      --duration ${params.duration} \
      --network ${params.network}`,
    { encoding: 'utf-8' }
  );
  return JSON.parse(result);
}
```

**Alloy direct call (fallback)**:

```rust
pub async fn configure_auction(
    &self,
    token_id: U256,
    params: &AuctionParams,
) -> Result<TxHash> {
    // Step 1: Approve Bazaar to transfer token
    let approve = approveCall {
        to: self.bazaar_address,
        tokenId: token_id,
    };
    self.provider
        .send_transaction(approve.encode(), self.series_address)
        .await?
        .receipt()
        .await?;

    // Step 2: Configure the auction
    let auction_type = match params.auction_type {
        AuctionType::Reserve => RESERVE_AUCTION_BYTES32,
        AuctionType::Scheduled => SCHEDULED_AUCTION_BYTES32,
        _ => RESERVE_AUCTION_BYTES32,
    };

    let config = configureAuctionCall {
        auctionType: auction_type,
        originContract: self.series_address,
        tokenId: token_id,
        startingAmount: params.reserve_wei,
        currencyAddress: Address::ZERO,  // ETH
        lengthOfAuction: U256::from(params.duration_seconds),
        startTime: U256::ZERO,  // Reserve = 0 start time
        splitAddresses: vec![],
        splitRatios: vec![],
    };

    let tx = self.provider
        .send_transaction(config.encode(), self.bazaar_address)
        .await?;
    Ok(tx.receipt().await?.tx_hash)
}
```

### Bidding (Self-Appraisal Narcissus Mode)

```rust
pub async fn bid_on_own(&self, token_id: U256, amount: U256) -> Result<TxHash> {
    let bid = offerCall {
        _originContract: self.series_address,
        _tokenId: token_id,
        _currencyAddress: Address::ZERO,
        _amount: amount,
        _convertible: true,
    };
    let tx = self.provider
        .send_transaction(bid.encode(), self.bazaar_address)
        .await?;
    Ok(tx.receipt().await?.tx_hash)
}
```

---

## Network Strategy

### Base + Ethereum Deployment

| Network | Use Case | Gas Cost | Notes |
|---------|----------|----------|-------|
| **Base Sepolia** | Hackathon / testnet | Free | SuperRare contracts on testnet |
| **Base** | Standard dream mints | ~$0.01 per tx | Low cost for frequent mints |
| **Ethereum Mainnet** | High-value pieces (death masks, rare Mandalas) | ~$2--8 per tx | SuperRare's primary marketplace |

**Strategy**: Mint on Base by default. Bridge to Ethereum for high-value pieces if configured. The bridge decision is based on:

1. Death masks: always bridge to Ethereum (if `nft.network` includes Ethereum)
2. Phi-peak Mandalas with `ms_since_last_peak > 1_000_000`: bridge (rare pieces)
3. Crucible artworks: bridge (rare by definition)
4. Standard dreams: Base only (too frequent for Ethereum gas)

### Contract Addresses

| Contract | Network | Address |
|----------|---------|---------|
| SuperRare Bazaar | Ethereum Mainnet | Canonical mainnet address (from SuperRare docs) |
| SuperRare Bazaar | Base Sepolia | Testnet deployment address |
| Golem Series | Per-Golem | Deployed during `on_session_start` |
| USDC | Base | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |

---

## Revenue Distribution Contract

Revenue from primary sales and secondary royalties flows through a split:

| Recipient | Share | Address Source |
|-----------|-------|---------------|
| Golem wallet | 85% | `GolemState.wallet_address` |
| Protocol treasury | 10% | Config: `bardo.treasury_address` |
| Successor fund | 5% | Config: `oneirography.successor_fund_address` |

For death mask auctions (Golem is dead): the Golem wallet's share goes to the owner wallet instead.

The split is configured via the Bazaar's `splitAddresses` and `splitRatios` parameters during auction configuration. No custom split contract needed -- the Bazaar handles distribution natively.

---

## ERC-721 Metadata Standard

All Oneirography NFTs follow the standard ERC-721 metadata schema with Oneirography-specific extensions:

### Base Fields

```json
{
  "name": "...",
  "description": "...",
  "image": "ipfs://Qm...",
  "animation_url": "ipfs://Qm...",
  "external_url": "https://bardo.run/golem/{golem_id}/...",
  "attributes": [...]
}
```

### Oneirography Extension Attributes

These attributes appear across all Oneirography NFT types:

| Attribute | Type | Present On | Description |
|-----------|------|-----------|-------------|
| `Golem ID` | string | All | Source Golem identifier |
| `Generation` | number | All | Golem generation number |
| `Behavioral Phase` | string | All | Phase at mint time |
| `Pleasure` | number | All | PAD P at mint time |
| `Arousal` | number | All | PAD A at mint time |
| `Dominance` | number | All | PAD D at mint time |
| `Plutchik Emotion` | string | All | Emotion label at mint time |
| `Ticks Alive` | number | All | Golem age at mint time |
| `Soul Encoded` | boolean | All (if stego enabled) | Whether soul data is embedded |
| `Generation Seed` | number | All (if Venice) | Reproducibility seed |
| `Is Death Mask` | boolean | All | Death mask flag |
| `Type` | string | Extended forms | "Phi-Peak Mandala", "Crucible", etc. |

### Type-Specific Attributes

See individual specs for type-specific metadata:
- Dream journals: `01-dream-journals.md`
- Death masks: `02-death-masks.md`
- Extended forms: `05-extended-forms.md`

---

## Configuration Reference

### Integration with the Config System

`[oneirography]` is a top-level TOML section, following the same pattern as `[custody]`, `[safety]`, and `[trading]` (from `07-tools/20-config.md`). It maps to `pub oneirography: Option<OneirographyConfig>` in `GolemExtendedManifest` (`01-golem/06-creation.md`). Configurable at golem creation (wizard, CLI, API) or in `golem.toml`.

### OneirographyConfig

```rust
pub struct OneirographyConfig {
    /// Master switch. If false, extension does not register.
    /// Default: false.
    pub enabled: bool,

    /// Which features are active. All opt-in. Default: empty.
    pub enable_features: Vec<OneirographyFeature>,

    /// Art spend as fraction of inference_allocation_fraction budget.
    /// Default: 0.10 (10% of inference allocation).
    pub art_budget_fraction: f64,

    /// Per-event hard ceilings.
    pub max_per_dream_usd: f64,       // default: 0.20
    pub max_per_phi_peak_usd: f64,    // default: 0.15
    pub max_per_crucible_usd: f64,    // default: 0.25
    pub max_per_death_mask_usd: f64,  // default: 8.00
    pub max_daily_art_spend_usd: f64, // default: 5.00

    /// NFT infrastructure.
    pub nft: Option<NftConfig>,

    /// Image generation providers.
    pub image_gen: ImageGenConfig,

    /// Bankr LLM gateway for art prompts.
    pub bankr_art: Option<BankrArtConfig>,

    /// Self-appraisal sub-config.
    pub self_appraisal: SelfAppraisalConfig,
}
```

### NftConfig

```rust
pub struct NftConfig {
    /// Deployed Series contract. If None, deploy on first mint.
    pub series_contract: Option<Address>,
    /// SuperRare Bazaar address. Defaults to canonical address.
    pub bazaar_contract: Option<Address>,
    /// Network for NFT operations. Default: base-sepolia.
    pub network: NftNetwork, // Ethereum | Base | BaseSepolia
    /// Secondary royalty in basis points. Default: 1000 (10%).
    pub royalty_bps: u16,
}
```

### ImageGenConfig

```rust
pub struct ImageGenConfig {
    /// Provider priority. First one with a configured API key is used.
    pub provider_priority: Vec<ImageProvider>, // default: ["venice", "stablestudio"]
    pub venice: Option<VeniceImageConfig>,
    pub stablestudio: Option<StableStudioConfig>,
}

pub struct VeniceImageConfig {
    // API key via BARDO_VENICE_IMAGE_API_KEY env var (never in TOML).
    pub model_standard: String,       // cost-effective, frequent dreams
    pub model_hq: String,             // high-novelty and terminal-phase dreams
    pub model_fast: String,           // high-arousal dreams (low cfg, dreamlike)
    pub model_death_mask: String,     // maximum quality, Thaler noise A/B
    pub variants_standard: u8,        // default: 3
    pub variants_death_mask: u8,      // default: 1
}

pub struct StableStudioConfig {
    // No additional config fields. Uses x402 on Base from golem's operational wallet.
}
```

### BankrArtConfig

```rust
pub struct BankrArtConfig {
    // API key via BARDO_BANKR_ART_API_KEY env var.
    pub wallet_id: String,
    pub max_dream_prompt_usd: f64,      // default: 0.02
    pub max_death_mask_prompt_usd: f64, // default: 0.15
}
```

### SelfAppraisalConfig

```rust
pub struct SelfAppraisalConfig {
    pub allow_bids: bool,        // default: false
    pub allow_burns: bool,       // default: false
    pub max_bid_fraction: f64,   // default: 0.02
}
```

---

## Capability Tier Requirements (Complete)

| Operation | Trust Tier | Capability Token |
|-----------|-----------|-----------------|
| Read Bazaar state (prices, bids) | ReadTool | None required |
| Read pheromone field | ReadTool | None required |
| Mint NFT via sidecar | WriteTool | `Capability<MintTool>` -- consumed |
| Configure auction on Bazaar | WriteTool | `Capability<ConfigAuctionTool>` |
| Place bid on own NFT (Narcissus) | WriteTool | `Capability<BazaarBidTool>` |
| Write ArtImpression to Grimoire | WriteTool | `Capability<GrimoireWriteTool>` |
| Burn NFT (Regret mode) | PrivilegedTool | `Capability<BurnTool>` + owner approval |

`WriteTool` capabilities are consumed (moved) on use. `PrivilegedTool` operations require pre-approval in the `PolicyCage`.

> **Cross-ref**: Trust tier system in `07-tools/01-architecture.md`

---

## SuperRare Contract Interfaces

### Series Contract (ERC-721)

| Function | Signature | Used By |
|----------|-----------|---------|
| `addNewToken` | `addNewToken(string _uri) -> uint256` | Minting |
| `approve` | `approve(address to, uint256 tokenId)` | Pre-auction approval |
| `burn` | `burn(uint256 tokenId)` | Regret mode |
| `updateTokenMetadata` | `updateTokenMetadata(uint256 tokenId, string _uri)` | Curator mode ratings |
| `tokenURI` | `tokenURI(uint256 tokenId) -> string` | Gallery data sourcing |
| `ownerOf` | `ownerOf(uint256 tokenId) -> address` | Self-appraisal checks |

### Bazaar Contract

| Function | Signature | Used By |
|----------|-----------|---------|
| `configureAuction` | `configureAuction(bytes32, address, uint256, uint256, address, uint256, uint256, address[], uint8[])` | Auction setup |
| `offer` | `offer(address, uint256, address, uint256, bool)` | Narcissus bidding |
| `setSalePrice` | `setSalePrice(address, uint256, uint256, address)` | Fixed-price Mandalas |

### Auxiliary Contracts

| Contract | Purpose |
|----------|---------|
| MarketplaceSettings | Fee configuration, approved token registry |
| ApprovedTokenRegistry | Which ERC-20 tokens are valid for bids |
| RarityPool | $RARE staking and curation rewards |

---

## Cross-References

| Document | Relevance |
|----------|-----------|
| `07-tools/01-architecture.md` | The TypeScript sidecar architecture and SidecarClient JSON-RPC pattern that the Rare Protocol CLI bridge extends for NFT operations |
| `01-golem/06-creation.md` | GolemExtendedManifest and creation config where Oneirography settings are first specified during Golem provisioning |
| `07-tools/20-config.md` | TOML config patterns and validation rules that OneirographyConfig follows for golem.toml integration |
| `01-golem/08-funding.md` | SustainabilityMetrics and budget computation determining how much the Golem can spend on minting gas and image generation |
| `21-integrations/02-venice.md` | Venice API for zero-retention image generation, the primary provider for all Oneirography image creation |
| `21-integrations/03-bankr.md` | Bankr wallet and LLM gateway where NFT auction revenue accumulates and funds art prompt inference |
