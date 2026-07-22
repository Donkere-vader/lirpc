import { LiRpcSerdeError } from "./error.js";

/**
 * Encodes/decodes messages to and from a transport's native frame type `F`.
 * Mirrors `lirpc_rs_client::serializers::Serializer<F>`.
 *
 * Rust splits this into `BytesSerializer`/`StringSerializer` purely to abstract over the frame
 * type produced by each transport (`Bytes` for TCP, `String` for WebSocket). WebSocket is the
 * only transport implemented here, so only a JSON/string serializer exists today; a future TCP
 * transport would supply its own `Serializer<Uint8Array>`.
 */
export interface Serializer<F> {
  serialize(message: unknown): F;
  deserialize(raw: F): unknown;
}

/** JSON-over-string serializer, used by `WebsocketTransport`. */
export class JsonStringSerializer implements Serializer<string> {
  serialize(message: unknown): string {
    try {
      return JSON.stringify(message);
    } catch (cause) {
      throw new LiRpcSerdeError("Failed to serialize message to JSON", cause);
    }
  }

  deserialize(raw: string): unknown {
    try {
      return JSON.parse(raw);
    } catch (cause) {
      throw new LiRpcSerdeError("Failed to deserialize message from JSON", cause);
    }
  }
}

export const jsonStringSerializer = new JsonStringSerializer();
