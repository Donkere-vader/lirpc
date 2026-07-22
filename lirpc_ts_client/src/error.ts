/**
 * Base class for all errors raised by the LiRPC client.
 * Mirrors the variants of `lirpc_rs_client::error::Error`.
 */
export class LiRpcError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = new.target.name;
  }
}

/** A frame could not be encoded/decoded as JSON, or didn't match the expected envelope shape. */
export class LiRpcSerdeError extends LiRpcError {
  constructor(message: string, cause?: unknown) {
    super(message, { cause });
  }
}

/** The underlying WebSocket connection failed to connect or errored while open. */
export class LiRpcWebsocketError extends LiRpcError {
  constructor(message: string, cause?: unknown) {
    super(message, { cause });
  }
}

/** The server responded with `res: "err"`. Mirrors `Error::Server { error, detail }`. */
export class LiRpcServerError extends LiRpcError {
  constructor(
    readonly errorType: string,
    readonly detail: string,
  ) {
    super(`ServerError: ${errorType}: ${detail}`);
  }
}

/**
 * The connection closed while this call was still pending.
 *
 * Deviation from `lirpc_rs_client`: the Rust client has no equivalent behavior — a pending
 * `Call` simply hangs forever if the connection dies before a response arrives. Rejecting
 * pending calls on close avoids leaving unresolved promises around in a GC'd, promise-based
 * environment.
 */
export class LiRpcConnectionClosedError extends LiRpcError {
  constructor() {
    super("Connection closed while a call was still pending");
  }
}
