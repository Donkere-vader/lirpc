import asyncio
import json

import websockets


async def run_client():
    uri = "ws://127.0.0.1:5000"
    async with websockets.connect(uri) as websocket:
        try:
            # Send one message
            headers = {
                "method": "do_something",
                "id": 0,
            }
            payload = {"name": "Cas"}
            await websocket.send(f"{json.dumps(headers)}\n\n{json.dumps(payload)}")
            print(f"Sent with headers: {headers} payload: {payload}")

            headers = {
                "method": "do_something_twice",
                "id": 1,
            }
            payload = {"name": "Cas2"}
            await websocket.send(f"{json.dumps(headers)}\n\n{json.dumps(payload)}")
            print(f"Sent with headers: {headers} payload: {payload}")

            # Continuously receive and print messages
            async for message in websocket:
                split_message = message.splitlines()
                print(
                    f"Received headers: {split_message[0]} payload: {split_message[2]}"
                )
        except asyncio.CancelledError:
            print("\nClosing websocket...")
            await websocket.close()
            print("Websocket closed.")
            raise


if __name__ == "__main__":
    try:
        asyncio.run(run_client())
    except KeyboardInterrupt:
        print("\nExiting...")
