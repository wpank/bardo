// Bardo tools-ts sidecar - JSON-RPC 2.0 server over Unix domain socket.
//
// Spawned by golem-tools at startup. Provides Uniswap V3/V4 concentrated
// liquidity math that has no mature Rust equivalent.
//
// Implemented by: Plan 26+ (golem-tools integration)
// This file is a stub. Plan 26 implements the actual tool handlers.

import * as net from "node:net";
import * as fs from "node:fs";
import type { JsonRpcRequest, JsonRpcResponse } from "./types.js";

const SOCKET_PATH = process.env["BARDO_SIDECAR_SOCKET"] ?? "/tmp/bardo-tools.sock";

if (fs.existsSync(SOCKET_PATH)) {
  fs.unlinkSync(SOCKET_PATH);
}

const server = net.createServer((socket) => {
  let buffer = "";

  socket.on("data", (chunk) => {
    buffer += chunk.toString();
    const lines = buffer.split("\n");
    buffer = lines.pop() ?? "";

    for (const line of lines) {
      if (!line.trim()) {
        continue;
      }

      try {
        const request = JSON.parse(line) as JsonRpcRequest;
        const response = handleRequest(request);
        socket.write(`${JSON.stringify(response)}\n`);
      } catch {
        const errorResponse: JsonRpcResponse = {
          jsonrpc: "2.0",
          id: "unknown",
          error: { code: -32700, message: "Parse error" },
        };
        socket.write(`${JSON.stringify(errorResponse)}\n`);
      }
    }
  });

  socket.on("error", (err) => {
    console.error("Socket error:", err.message);
  });
});

function handleRequest(request: JsonRpcRequest): JsonRpcResponse {
  return {
    jsonrpc: "2.0",
    id: request.id,
    error: {
      code: -32601,
      message: `Method not found: ${request.method} (sidecar not yet implemented - see Plan 26)`,
    },
  };
}

server.listen(SOCKET_PATH, () => {
  console.log(`Bardo tools-ts sidecar listening on ${SOCKET_PATH}`);
});

process.on("SIGTERM", () => {
  server.close(() => process.exit(0));
});

process.on("SIGINT", () => {
  server.close(() => process.exit(0));
});
