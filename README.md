WMBP (Websocket multiplayer backend proxy)
=========================================

This is a pet weekend project to learn & enjoy Rust. Use at your own risk.

## Introduction

WMBP is a project is to help more party games to sprout by providing a backend implementation.
The base implementation is a one (*Game*) to many (*Clients*) connection communication pattern, which is coordinated by the *Server*, allowing **serverless party games**.

The *Game* connects to the *Server* with a websocket and creates a room. Once the room is created, the *Game* will receive the **Room Code** which will be used by *Clients* to connect.

While the *Game* has a mandatory communication pattern with the *Server*, the *Clients* have a **seemingly direct connection to the *Game***, both in **binary** and **text**.

![Server example](doc/multiplayer_proxy.png)

## Usage

5. Spin this **Server**, `cargo run --release`
5. **Game** creates a websocket connection to this server to create a game `new WebSocket("ws://127.0.0.1:8081/CREATE")`
5. **Game** sends Prepare with a certain amount of players.
5. **Server** sends to **Game** the *Room Code* and refuses players if there are too many
5. **Clients** creates a websocket connection to this server to create a game `new WebSocket("ws://127.0.0.1:8081/ROOM_CODE")`
5. **Game** sends Start once enough players are connected.
5. **Game** sends data to **Clients** with To and ToStr messages.
5. **Clients** sends data to **Game** with plain text or arraybuffer (for binary format).
5. **Game** receives data from **Clients** with From and FromStr messages.

![The game flow of the server](doc/flow.png)

## In depth

Game type:
- `http://127.0.0.1:8081/ROOM` returns (in text/plain) the game type (started with Prepare)

Then endpoint of the websocket server is defining if you are a Host client (Game) or a Player client by the websocket you created.
- `ws://127.0.0.1:8081/CREATE` creates a new game
- `ws://127.0.0.1:8081/ROOM` connects to a room

### Messages as a Game (host)

The messages for the *Game* are **CBOR** encoded with the following format: `{ "cmd": "snake_case_command", "data1": 1, "data2": "data2"}`

For rust users, just take a look at the enum [src/comm.rs#Commands](src/comm.rs).
For Javascript users:
- **\> Prepare**: `{"cmd": "prepare", "max_players": 8, "name": "test"}` # Prepares the game with the maximum number of clients
- **< PrepareReply**: `{"cmd": "prepare_reply", "key": "ROOM"}` # On successful game creation, provides the ROOM key
- **< PlayerJoined**: `{"cmd": "player_joined", "player": 12}` # A new player joined
- **< PlayerLeft**: `{"cmd": "player_left", "player": 12}` # A player left
- **\> Start**: `{"cmd": "start"}` # Starts the game, prevents the clients to connect from this point on.
- **< State**: `{"cmd": "state", "players": [5,2,3], "max_players": 8, "accept_conns": true}` # Provides information about the game, players connected, etc.
- **\> Kick**: `{"cmd": "kick", "player": 5}` # Kicks player with id 5 (from the State message)
- **< Stop**: `{"cmd": "stop"}` # Disconnect everybody
- **\> To**: `{"cmd": "to", to: [2], "data": [1,2,3]}` # Sends binary data to the user 1
- **\> ToStr**: `{"cmd": "to_str", to: [3, 5], "data": "some string"}` # Sends text data to the user 3 and 5
- **< From**: `{"cmd": "from", "from": 2, "data": [1,2,3]}` # Received when user 2 sent binary data
- **< FromStr**: `{"cmd": "from", "from": 5, "data": "some string"}` # Received when user 5 sent string data

### Messages as a client

The *Client* has no specific message. Sending text (Text type for the websocket), the message will be transferred with `FromStr` to the *Game*, while sending binary data (such as CBOR encore data or images) will be forwarded with `From` to the *Game*.

Whenever *Game* sends data with `To` and `ToStr`, only the data will be forwarded to the client (as everything else would be redundant) as binary or text.

This means the *Client* has a connection that seems to be directly to the game.

### Optional features

#### tls

In a wim, I quickly made a TLS feature based on `rustls`, following the example in [hyper-rustls](https://github.com/rustls/hyper-rustls/tree/main/examples). Only later on I understood this was the responsibilty of the cloud service (or nginx or other) in most cases.
I have no clue what I am doing 🙈


### Deploy

#### Render.com

Either you try the blueprint (render.yaml provided) or you create a new app with:
- Set the repo to this one
- Define the PORT environment variable as `10000`, this fasten the spinup of the machines
- Set the build command as `cargo build --release`
- Set the run command as `cargo run --release 0.0.0.0:10000`

#### Google Cloud

Create a new project, and, using the PROJECT_ID you created:

```bash
gcloud config set project PROJECT_ID
gcloud run deploy
```

You can [Delete your ressources here](https://console.cloud.google.com/iam-admin/projects?utm_source=cloud.google.com)
