import assert from "node:assert/strict";
import { test } from "node:test";
import { isLiRpcServerErrorPayload, isOk, toLiRpcResponse } from "./lirpc_message.js";

test("isOk treats an absent res as ok", () => {
  assert.equal(isOk({ id: 1 }), true);
  assert.equal(isOk({ id: 1, res: "ok" }), true);
  assert.equal(isOk({ id: 1, res: "err" }), false);
});

test("toLiRpcResponse accepts a well-formed success envelope with omitted res/payload", () => {
  const response = toLiRpcResponse({ headers: { id: 1 } });
  assert.deepEqual(response, { headers: { id: 1, res: undefined }, payload: undefined });
});

test("toLiRpcResponse accepts a well-formed envelope with payload and res", () => {
  const response = toLiRpcResponse({ headers: { id: 2, res: "err" }, payload: { error: "e", detail: "d" } });
  assert.deepEqual(response, {
    headers: { id: 2, res: "err" },
    payload: { error: "e", detail: "d" },
  });
});

test("toLiRpcResponse rejects frames missing a numeric header id", () => {
  assert.equal(toLiRpcResponse({ headers: {} }), null);
  assert.equal(toLiRpcResponse({ headers: { id: "1" } }), null);
});

test("toLiRpcResponse rejects frames with an invalid res value", () => {
  assert.equal(toLiRpcResponse({ headers: { id: 1, res: "maybe" } }), null);
});

test("toLiRpcResponse rejects non-object frames", () => {
  assert.equal(toLiRpcResponse("not an object"), null);
  assert.equal(toLiRpcResponse(null), null);
  assert.equal(toLiRpcResponse({ headers: "not an object" }), null);
});

test("isLiRpcServerErrorPayload validates the {error, detail} shape", () => {
  assert.equal(isLiRpcServerErrorPayload({ error: "e", detail: "d" }), true);
  assert.equal(isLiRpcServerErrorPayload({ error: "e" }), false);
  assert.equal(isLiRpcServerErrorPayload(null), false);
  assert.equal(isLiRpcServerErrorPayload("nope"), false);
});
