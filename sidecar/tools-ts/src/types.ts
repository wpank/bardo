// JSON-RPC 2.0 types for the Bardo tools-ts sidecar.
// The Rust golem-tools crate connects to this over a Unix domain socket.

export interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params: unknown;
}

export interface JsonRpcResponse<T = unknown> {
  jsonrpc: "2.0";
  id: number | string;
  result?: T;
  error?: JsonRpcError;
}

export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

// Method registry - filled in by Plan 26+ (golem-tools implementation)
export type SidecarMethod =
  | "uniswap_v3_quote"
  | "uniswap_v3_position_amounts"
  | "uniswap_v4_quote"
  | "uniswap_route_optimal";
