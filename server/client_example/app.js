class Game {
    constructor() {
        this.textarea = document.getElementById('console_output');
        this.roomid = document.getElementById('roomid');
        this.username = document.getElementById('username');
        this.data = document.getElementById('data');
        this.ws = undefined;
    }

    connect() {
        this.conn_start("CONNECT");
    }

    create() {
        this.conn_start("CREATE");
    }

    conn_start(type) {
        if (this.ws != undefined && (this.ws.readyState == 2 || this.ws.readyState == 3)) {
            this.ws.onclose = undefined;
            this.ws.onerror = undefined;
            this.ws.onopen = undefined;
            this.ws.onconnect = undefined;
            this.ws.onmessage = undefined;
            this.ws.close();
            this.ws = undefined;
        }
        if (this.ws == undefined) {
            this.log("[CONNECTING] to " + roomid.value + " as " + username.value);
            this.ws = new WebSocket("ws://127.0.0.1:8081/" + type + "/" + roomid.value);
            this.ws.binaryType = "arraybuffer";
            this.ws.onclose = (a) => { this.log("[CLOSED] Code: " + a.code + " Reason: \"" + a.reason + "\""); }
            this.ws.onerror = (a) => { this.log("[ERROR]"); }
            this.ws.onopen = (a) => { this.log("[OPENED]"); }
            this.ws.onconnect = (a) => { this.log("[CONNECTED]"); };
            this.ws.onmessage = (a) => {
                let msg = CBOR.decode(a.data);
                if (msg.cmd == "prepare_reply") {
                    this.on_prepare_reply(msg);
                } else if (msg.cmd == "from") {
                    this.on_player_data(msg);
                } else if (msg.cmd == "stop") {
                    this.on_stop(msg);
                } else if (msg.cmd == "error") {
                    this.on_error(msg);
                } else if (msg.cmd == "state") {
                    this.on_state(msg);
                } else {
                    this.log("[MESSAGE IN] Unknown message: ", JSON.stringify(CBOR.decode(a.data)));
                }
            };
        }
        else {
            this.log("[CONNECTING] The websocket is already connecting.");
        }
    }

    log(logdata) {
        this.textarea.value += (logdata + "\n");
        this.textarea.scrollTop = this.textarea.scrollHeight;
    }

    send(val) {
        if (this.ws != undefined && this.ws.readyState == 1) {
            this.log("[MESSAGE OUT] Data send: " + JSON.stringify(val));
            this.ws.send(CBOR.encode(val));
        } else {
            this.log("[MESSAGE OUT] The websocket is not (Yet?) connected.");
        }
    }

    send_field() {
        this.send(this.data.value);
    }

    prepare(max_players) {
        this.send({
            cmd: "prepare",
            max_players,
        });
    }

    start() {
        this.send({
            cmd: "start",
        })
    }

    kick(player) {
        this.send({
            player,
            cmd: "kick",
        })
    }

    stop() {
        this.send({
            cmd: "stop",
        })
    }

    // to is an array of user id
    to(to, data) {
        this.send({
            to,
            data,
        })
    }

    on_prepare_reply(data) {
        this.log("[PREPARE_REPLY] Game key: " + data.key);
    }
    on_player_data(data) {
        this.log("[PLAYER_DATA] " + data.from + " sent: " + data.data);
    }
    on_stop(data) {
        this.log("[STOP]");
    }
    on_error(data) {
        this.log("[ERROR] reason: " + data.reason);
    }
    on_state(data) {
        this.log("[STATE] players: " + JSON.stringify(data));
    }
};


var game = new Game();
