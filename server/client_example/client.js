class Client {
    constructor() {
        this.textarea = document.getElementById('console_output');
        this.roomid = document.getElementById('roomid');
        this.username = document.getElementById('username');
        this.data = document.getElementById('data');
        this.ws = undefined;
    }

    connect() {
        this.conn_start(this.roomid.value);
    }

    conn_start(game) {
        if (this.ws != undefined && (this.ws.readyState == 2 || this.ws.readyState == 3)) {
            this.ws.onclose = undefined;
            this.ws.onerror = undefined;
            this.ws.onopen = undefined;
            this.ws.onmessage = undefined;
            this.ws.close();
            this.ws = undefined;
        }
        if (this.ws == undefined) {
            this.log("[CONNCETING] Joining the game: " + game);
            this.ws = new WebSocket("ws://127.0.0.1:8081/" + game);
            this.ws.binaryType = "arraybuffer";
            this.ws.onclose = (a) => { this.log("[CLOSED] Code: " + a.code + " Reason: \"" + a.reason + "\"");}
            this.ws.onerror = (a) => { this.log("[ERROR]"); }
            this.ws.onopen = (a) => { this.log("[OPENED]"); }
            this.ws.onmessage = (a) => {
                console.log(a);
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
            this.ws.send(val);
        } else {
            this.log("[MESSAGE OUT] The websocket is not (Yet?) connected.");
        }
    }

    send_cbor(val) {
        this.send(CBOR.encode(val));
    }

    send_field() {
        this.send(this.data.value);
    }

    on_str(data) {
        this.log("[DATA STR] reason: " + data);
    }

    on_bin(data) {
        this.log("[DATA BIN] players: " + JSON.stringify(data));
    }
};


var game = new Client();
