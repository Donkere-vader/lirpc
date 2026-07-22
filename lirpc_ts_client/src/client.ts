import { LiRpcConnectionClosedError, LiRpcSerdeError, LiRpcServerError } from "./error.js";
import {
  isLiRpcServerErrorPayload,
  isOk,
  toLiRpcResponse,
  type LiRpcRequest,
} from "./lirpc_message.js";
import type { Transport } from "./transport/index.js";
import { WebsocketTransport } from "./transport/websocket.js";

interface PendingCall {
  resolve(payload: unknown): void;
  reject(error: unknown): void;
}

/**
 * LiRPC client, generic over its transport's frame type `F`. Mirrors
 * `lirpc_rs_client::Client<T, F>`.
 *
 * Unlike the Rust client, `call()` returns a `Promise<R>` directly instead of a `Call<R>` handle
 * that must be separately `.resolve()`d — that two-phase shape in Rust exists only because of
 * ownership constraints that don't apply here.
 */
export class Client<F> {
  private idCounter = 0;

  private constructor(
    private readonly transport: Transport<F>,
    private readonly pending: Map<number, PendingCall>,
  ) {}

  /** Connects over WebSocket. Mirrors `Client::new_websocket`. */
  static async connectWebsocket(url: string): Promise<Client<string>> {
    const pending = new Map<number, PendingCall>();
    let transport!: WebsocketTransport;
    transport = await WebsocketTransport.connect(
      url,
      (frame) => Client.handleIncomingFrame(pending, transport, frame),
      () => Client.rejectAllPending(pending, new LiRpcConnectionClosedError()),
    );
    return new Client(transport, pending);
  }

  /** `u32` wrapping increment, mirrors `Client::get_new_request_id`. */
  private getNewRequestId(): number {
    this.idCounter = (this.idCounter + 1) >>> 0;
    return this.idCounter;
  }

  private static handleIncomingFrame<F>(
    pending: Map<number, PendingCall>,
    transport: Transport<F>,
    frame: F,
  ): void {
    let decoded: unknown;
    try {
      decoded = transport.serializer.deserialize(frame);
    } catch {
      return;
    }

    const response = toLiRpcResponse(decoded);
    if (response === null) {
      return;
    }

    const call = pending.get(response.headers.id);
    if (call === undefined) {
      console.warn(`Received message with no listener waiting (id ${response.headers.id})`);
      return;
    }
    pending.delete(response.headers.id);

    if (!isOk(response.headers)) {
      if (isLiRpcServerErrorPayload(response.payload)) {
        call.reject(new LiRpcServerError(response.payload.error, response.payload.detail));
      } else {
        call.reject(new LiRpcSerdeError("Server error response did not match expected shape"));
      }
      return;
    }

    call.resolve(response.payload);
  }

  private static rejectAllPending(pending: Map<number, PendingCall>, error: unknown): void {
    for (const call of pending.values()) {
      call.reject(error);
    }
    pending.clear();
  }

  /**
   * Calls a remote function by name. Mirrors `Client::call` + `Call::resolve` collapsed into a
   * single `Promise`.
   */
  call<M, R>(functionName: string, payload?: M): Promise<R> {
    const id = this.getNewRequestId();
    const request: LiRpcRequest<M> = {
      headers: { id, function: functionName },
      payload: payload ?? null,
    };

    return new Promise<R>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (payload: unknown) => void, reject });
      this.transport.send(request).catch((error) => {
        this.pending.delete(id);
        reject(error);
      });
    });
  }
}
