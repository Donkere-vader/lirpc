// Mirrors lirpc_rs_client/examples/greeter_client.rs and client_examples/greeter/greeter_client.

import { Client } from "../src/index.js";

interface GreetingRequest {
  name: string;
}

interface GreetingResponse {
  msg: string;
}

async function greet(client: Client<string>, request: GreetingRequest): Promise<GreetingResponse> {
  return client.call<GreetingRequest, GreetingResponse>("greet", request);
}

const client = await Client.connectWebsocket("ws://127.0.0.1:5000");

const response = await greet(client, { name: "Cas" });

console.log(`Server said: ${response.msg}`);
