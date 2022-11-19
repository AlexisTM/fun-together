const CLIENT_TYPES = {
    Unknown: 0,
    Host: 1,
    Player: 2,
};
var last_msg = undefined;
class Game {
    constructor() {
        this.ws = undefined;
        this.type = CLIENT_TYPES.Unknown;
    }

    is_host() {
        return this.type == CLIENT_TYPES.Host;
    }

    is_player() {
        return this.type == CLIENT_TYPES.Player;
    }

    is_unknown() {
        return this.type == CLIENT_TYPES.Unknown;
    }

    join(roomid) {
        if (roomid.length == 4) {
            if (this.conn_start(roomid)) {
                this.type = CLIENT_TYPES.Player;
                return true;
            }
        }
        return false;
    }

    create() {
        if (this.conn_start("CREATE")) {
            this.type = CLIENT_TYPES.Host;
            return true;
        }
        return false;
    }

    conn_start(game) {
        try {
            if (this.ws != undefined && (this.ws.readyState == 2 || this.ws.readyState == 3)) {
                this.ws.onclose = undefined;
                this.ws.onerror = undefined;
                this.ws.onopen = undefined;
                this.ws.onmessage = undefined;
                this.ws.close();
                this.ws = undefined;
            }
            if (this.ws == undefined) {
                let type = undefined;
                if (game == 'CREATE') {
                    this.on_log('[CONNECTING] Creating a game');
                } else {
                    this.on_log('[CONNCETING] Joining the game: ' + game);
                }
                this.ws = new WebSocket('ws://127.0.0.1:8081/' + game);
                this.ws.binaryType = 'arraybuffer';
                this.ws.onclose = (a) => { this.on_ws_close(a); this.on_log('[CLOSED] Code: ' + a.code + ' Reason: \"' + a.reason + '\"'); }
                this.ws.onerror = (a) => { this.on_log('[ERROR]'); }
                this.ws.onopen = (a) => { this.on_log('[OPENED]'); }
                this.ws.onmessage = (a) => {
                    last_msg = a;
                    if (this.is_host()) {
                        let msg = CBOR.decode(a.data);
                        this.on_log("[MESSAGE IN] " + JSON.stringify(msg));
                        if (msg.cmd == 'prepare_reply') {
                            this.on_prepare_reply(msg);
                        } else if (msg.cmd == 'from') {
                            this.on_player_data(msg);
                        } else if (msg.cmd == 'from_str') {
                            this.on_player_data(msg);
                        } else if (msg.cmd == 'stop') {
                            this.on_stop(msg);
                        } else if (msg.cmd == 'error') {
                            this.on_error(msg);
                        } else if (msg.cmd == 'state') {
                            this.on_state(msg);
                        } else {
                            this.on_log('[MESSAGE IN] Unknown message: ' + a.data);
                        }
                    } else {
                        if (typeof (a.data) == 'string') {
                            this.on_host_str(a.data);
                        } else {
                            this.on_host_bin(a.data);
                        }
                    }
                };
                return true;
            }
            else {
                this.on_log('[CONNECTING] The websocket is already connecting.');
            }
            return false;
        } catch (error) {
            console.log(error);
            return false;
        }
    }

    send(val) {
        if (this.ws != undefined && this.ws.readyState == 1) {
            this.ws.send(val);
        } else {
            this.on_log('[MESSAGE OUT] The websocket is not (Yet?) connected.');
        }
    }

    send_cbor(val) {
        this.on_log('[MESSAGE OUT] Data sent: ' + JSON.stringify(val));
        this.send(CBOR.encode(val));
    }

    prepare(max_players) {
        this.send_cbor({
            cmd: 'prepare',
            max_players,
        });
    }

    start() {
        this.send_cbor({
            cmd: 'start',
        })
    }

    kick(player) {
        this.send_cbor({
            cmd: 'kick',
            player,
        })
    }

    stop() {
        this.send_cbor({
            cmd: 'stop',
        })
    }

    // to is an array of user id
    // Data is a raw data (an array or CBOR encoded object)
    to(to, data) {
        // CBOR fails to serialize bytearray (its output)
        // serde (ciborium) fails to deserialize the output of this CBOR using Uint8Array
        // Thus converting it all the way to an array in a triple copy design patter.
        const typedArray = new Uint8Array(data);
        const array = [...typedArray];
        this.send_cbor({
            cmd: 'to',
            to,
            data: array, // CBOR cannot serialize its own output, and Uint8Array is not recognized Rust's side
        })
    }

    // to is an array of user id
    // Data will be cbor encoded
    to_cbor(to, data) {
        this.to(to, CBOR.encode(data))
    }

    // to is an array of user id
    // Data is a string
    to_str(to, data) {
        this.send_cbor({
            cmd: 'to_str',
            to,
            data,
        })
    }

    // Player callbacks
    on_host_bin(data) {
        this.on_log('[MESSAGE IN] ' + JSON.stringify(data));
    }
    on_host_str(data) {
        this.on_log('[MESSAGE IN] ' + data);
    }
    // Host callbacks
    on_prepare_reply(data) {
        this.on_log('[PREPARE_REPLY] Game key: ' + data.key);
    }
    on_player_data(data) {
        this.on_log('[PLAYER_DATA] ' + data.from + ' sent: ' + data.data);
    }
    on_stop(data) {
        this.on_log('[STOP]');
    }
    on_error(data) {
        this.on_log('[ERROR] reason: ' + data.reason);
    }
    on_state(data) {
        this.on_log('[STATE] players: ' + JSON.stringify(data));
    }

    // Configuration related
    on_log(logdata) { }
    on_ws_close(data) {
        this.on_log('[CLOSED] Code: ' + a.code + ' Reason: \"' + a.reason + '\"');
    }
};
