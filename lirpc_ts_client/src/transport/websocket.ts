import { LiRpcWebsocketError } from "../error.js";
import { jsonStringSerializer, type Serializer } from "../serializer.js";
import type { Transport } from "./index.js";

/**
 * WebSocket transport using the global `WebSocket` API (browser, or any runtime that provides
 * one). Mirrors `lirpc_rs_client::transport::websocket::Websocket`.
 */
export class WebsocketTransport implements Transport<string> {
  readonly serializer: Serializer<string> = jsonStringSerializer;

  private constructor(private readonly socket: WebSocket) {}

  /**
   * Connects to `url` and starts forwarding incoming text frames to `onMessage`. Binary/other
   * frame types are silently ignored, matching the Rust transport's handling of non-`Text`
   * websocket messages. `onClose` fires once, on either a `close` or a post-connect `error`
   * event.
   */
  static connect(
    url: string,
    onMessage: (frame: string) => void,
    onClose: () => void,
  ): Promise<WebsocketTransport> {
    return new Promise((resolve, reject) => {
      const socket = new WebSocket(url);

      socket.addEventListener("open", () => resolve(new WebsocketTransport(socket)), {
        once: true,
      });

      socket.addEventListener(
        "error",
        () => reject(new LiRpcWebsocketError(`Failed to connect to ${url}`)),
        { once: true },
      );

      socket.addEventListener("message", (event) => {
        if (typeof event.data === "string") {
          onMessage(event.data);
        }
      });

      socket.addEventListener("close", () => onClose(), { once: true });
    });
  }

  async send(message: unknown): Promise<void> {
    const raw = this.serializer.serialize(message);
    this.socket.send(raw);
  }
}
