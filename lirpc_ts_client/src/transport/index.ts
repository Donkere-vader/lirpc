import type { Serializer } from "../serializer.js";

/**
 * A pluggable transport, generic over its native frame type `F` (`string` for WebSocket,
 * `Uint8Array` for a future length-delimited TCP transport). Mirrors
 * `lirpc_rs_client::transport::Transport<F>`.
 *
 * Like the Rust trait, this only covers sending — each transport implementation is responsible
 * for reading its own socket and delivering decoded frames to the caller out-of-band (via the
 * `onMessage` callback passed to its own `connect`), rather than exposing a `recv` method here.
 */
export interface Transport<F> {
  readonly serializer: Serializer<F>;
  send(message: unknown): Promise<void>;
}
