/**
 * Wire types for LiRPC requests/responses. Mirrors `lirpc_rs_client::lirpc_message` /
 * `lirpc::lirpc_message`.
 *
 * Request:  {"headers":{"id":<u32>,"function":<string>},"payload":<P>|null}
 * Response: {"headers":{"id":<u32>,"res"?:"ok"|"err"},"payload"?:<P>}
 *
 * `res` is omitted by the server on success (absent means "ok"); `payload` is omitted on the
 * response when there is none.
 */

export interface LiRpcRequestHeaders {
  id: number;
  function: string;
}

export interface LiRpcRequest<P> {
  headers: LiRpcRequestHeaders;
  payload: P | null;
}

export type LiRpcResponseResult = "ok" | "err";

export interface LiRpcResponseHeaders {
  id: number;
  res?: LiRpcResponseResult;
}

export interface LiRpcResponse {
  headers: LiRpcResponseHeaders;
  payload?: unknown;
}

/** Mirrors `LiRpcServerError { error: String, detail: String }`. */
export interface LiRpcServerErrorPayload {
  error: string;
  detail: string;
}

export function isOk(headers: LiRpcResponseHeaders): boolean {
  return headers.res === undefined || headers.res === "ok";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Validates a deserialized frame (already run through a `Serializer.deserialize`) against the
 * expected `LiRpcResponse` envelope shape, or returns `null` if it doesn't match. Mirrors
 * `message_router` silently `continue`-ing on malformed frames instead of tearing down the
 * connection.
 */
export function toLiRpcResponse(value: unknown): LiRpcResponse | null {
  if (!isRecord(value) || !isRecord(value.headers)) {
    return null;
  }

  const { id, res } = value.headers;
  if (typeof id !== "number") {
    return null;
  }
  if (res !== undefined && res !== "ok" && res !== "err") {
    return null;
  }

  return {
    headers: { id, res },
    payload: "payload" in value ? value.payload : undefined,
  };
}

export function isLiRpcServerErrorPayload(value: unknown): value is LiRpcServerErrorPayload {
  return (
    isRecord(value) && typeof value.error === "string" && typeof value.detail === "string"
  );
}
