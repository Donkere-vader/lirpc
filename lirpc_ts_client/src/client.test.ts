import assert from "node:assert/strict";
import { test } from "node:test";
import { Client } from "./client.js";
import { LiRpcConnectionClosedError, LiRpcServerError } from "./error.js";

type Listener = (event: any) => void;

/** Minimal fake of the global `WebSocket` API, driven manually from tests. */
class FakeWebSocket {
  static instances: FakeWebSocket[] = [];

  private readonly listeners: Record<string, Listener[]> = {};
  readonly sent: string[] = [];

  constructor(readonly url: string) {
    FakeWebSocket.instances.push(this);
  }

  addEventListener(type: string, listener: Listener): void {
    (this.listeners[type] ??= []).push(listener);
  }

  send(data: string): void {
    this.sent.push(data);
  }

  open(): void {
    this.dispatch("open", {});
  }

  message(data: string): void {
    this.dispatch("message", { data });
  }

  close(): void {
    this.dispatch("close", {});
  }

  private dispatch(type: string, event: unknown): void {
    for (const listener of this.listeners[type] ?? []) {
      listener(event);
    }
  }
}

/** Installs `FakeWebSocket` as the global `WebSocket`, runs `fn`, then restores the original. */
async function withFakeWebsocket<T>(fn: () => Promise<T>): Promise<T> {
  const original = globalThis.WebSocket;
  FakeWebSocket.instances.length = 0;
  (globalThis as unknown as { WebSocket: unknown }).WebSocket = FakeWebSocket;
  try {
    return await fn();
  } finally {
    (globalThis as unknown as { WebSocket: unknown }).WebSocket = original;
  }
}

async function connectFakeClient(): Promise<{ client: Client<string>; socket: FakeWebSocket }> {
  const clientPromise = Client.connectWebsocket("ws://example.test");
  const socket = FakeWebSocket.instances.at(-1)!;
  socket.open();
  return { client: await clientPromise, socket };
}

test("call() sends the request envelope and resolves on a matching ok response", async () => {
  await withFakeWebsocket(async () => {
    const { client, socket } = await connectFakeClient();

    const callPromise = client.call<{ name: string }, { msg: string }>("greet", { name: "Cas" });

    assert.equal(socket.sent.length, 1);
    assert.deepEqual(JSON.parse(socket.sent[0]), {
      headers: { id: 1, function: "greet" },
      payload: { name: "Cas" },
    });

    socket.message(JSON.stringify({ headers: { id: 1 }, payload: { msg: "Hello Cas!" } }));

    assert.deepEqual(await callPromise, { msg: "Hello Cas!" });
  });
});

test("omitted payload is sent as null, not omitted", async () => {
  await withFakeWebsocket(async () => {
    const { client, socket } = await connectFakeClient();

    void client.call("ping");

    assert.deepEqual(JSON.parse(socket.sent[0]), {
      headers: { id: 1, function: "ping" },
      payload: null,
    });
  });
});

test("an err response rejects with LiRpcServerError carrying error/detail", async () => {
  await withFakeWebsocket(async () => {
    const { client, socket } = await connectFakeClient();

    const callPromise = client.call("login", { username: "cas" });
    socket.message(
      JSON.stringify({
        headers: { id: 1, res: "err" },
        payload: { error: "AuthFailure", detail: "bad credentials" },
      }),
    );

    await assert.rejects(callPromise, (error: unknown) => {
      if (!(error instanceof LiRpcServerError)) {
        throw new Error("expected a LiRpcServerError");
      }
      assert.equal(error.errorType, "AuthFailure");
      assert.equal(error.detail, "bad credentials");
      return true;
    });
  });
});

test("a malformed frame is silently ignored and does not affect pending calls", async () => {
  await withFakeWebsocket(async () => {
    const { client, socket } = await connectFakeClient();

    const callPromise = client.call<undefined, { msg: string }>("greet");

    socket.message("not json");
    socket.message(JSON.stringify({ no: "headers" }));
    socket.message(JSON.stringify({ headers: { id: "1" } }));

    socket.message(JSON.stringify({ headers: { id: 1 }, payload: { msg: "still works" } }));

    assert.deepEqual(await callPromise, { msg: "still works" });
  });
});

test("a response with an id nobody is waiting for is ignored (warns, doesn't throw)", async () => {
  const originalWarn = console.warn;
  const warnings: unknown[][] = [];
  console.warn = (...args: unknown[]) => warnings.push(args);

  try {
    await withFakeWebsocket(async () => {
      const { client, socket } = await connectFakeClient();

      socket.message(JSON.stringify({ headers: { id: 999 }, payload: { msg: "nobody asked" } }));
      assert.equal(warnings.length, 1);

      const callPromise = client.call<undefined, { msg: string }>("greet");
      socket.message(JSON.stringify({ headers: { id: 1 }, payload: { msg: "hi" } }));
      assert.deepEqual(await callPromise, { msg: "hi" });
    });
  } finally {
    console.warn = originalWarn;
  }
});

test("request ids wrap around like a u32 (wrapping_add semantics)", async () => {
  await withFakeWebsocket(async () => {
    const { client, socket } = await connectFakeClient();
    (client as unknown as { idCounter: number }).idCounter = 0xffffffff;

    void client.call("greet");

    assert.equal(JSON.parse(socket.sent[0]).headers.id, 0);
  });
});

test("closing the socket rejects any still-pending calls with LiRpcConnectionClosedError", async () => {
  await withFakeWebsocket(async () => {
    const { client, socket } = await connectFakeClient();

    const callPromise = client.call("greet", { name: "Cas" });
    socket.close();

    await assert.rejects(callPromise, LiRpcConnectionClosedError);
  });
});
