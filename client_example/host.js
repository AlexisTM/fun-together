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
        console.log(roomid)
        if (roomid.length == 4) {
            if (this.conn_start(roomid)) {
                this.type = CLIENT_TYPES.Player;
                console.log('started')
                return true;
            }
            console.log('good id but failed ')
        }
        console.log('bad id')
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
                this.log('[CONNECTING] Creating a game');
            } else {
                this.log('[CONNCETING] Joining the game: ' + game);
            }
            this.ws = new WebSocket('ws://127.0.0.1:8081/' + game);
            this.ws.binaryType = 'arraybuffer';
            this.ws.onclose = (a) => { this.log('[CLOSED] Code: ' + a.code + ' Reason: \"' + a.reason + '\"'); }
            this.ws.onerror = (a) => { this.log('[ERROR]'); }
            this.ws.onopen = (a) => { this.log('[OPENED]'); }
            this.ws.onmessage = (a) => {
                last_msg = a;
                if (this.is_host()) {
                    this.log('[MESSAGE IN] ' + a.data);
                    let msg = CBOR.decode(a.data);
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
                        this.log('[MESSAGE IN] Unknown message: ' + a.data);
                    }
                } else {
                    if (typeof (a) == 'string') {
                        this.on_host_str(a);
                    } else {
                        this.on_host_bin(a);
                    }
                }
            };
            return true;
        }
        else {
            this.log('[CONNECTING] The websocket is already connecting.');
        }
        return false;
    }

    log(logdata) {
        // this.textarea.value += (logdata + "\n");
        // this.textarea.scrollTop = this.textarea.scrollHeight;
    }

    send(val) {
        if (this.ws != undefined && this.ws.readyState == 1) {
            this.log('[MESSAGE OUT] Data send: ' + JSON.stringify(val));
            this.ws.send(val);
        } else {
            this.log('[MESSAGE OUT] The websocket is not (Yet?) connected.');
        }
    }

    send_cbor(val) {
        this.send(CBOR.encode(val))
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
    to(to, data) {
        this.send_cbor({
            cmd: 'to',
            to,
            data,
        })
    }

    // to is an array of user id
    to_str(to, data) {
        this.send_cbor({
            cmd: 'to_str',
            to,
            data,
        })
    }

    // Player callbacks
    on_host_bin(data) {
        this.log('[MESSAGE IN] ' + JSON.stringify(data));
    }
    on_host_str(data) {
        this.log('[MESSAGE IN] ' + data);
    }
    // Host callbacks
    on_prepare_reply(data) {
        this.log('[PREPARE_REPLY] Game key: ' + data.key);
    }
    on_player_data(data) {
        this.log('[PLAYER_DATA] ' + data.from + ' sent: ' + data.data);
    }
    on_stop(data) {
        this.log('[STOP]');
    }
    on_error(data) {
        this.log('[ERROR] reason: ' + data.reason);
    }
    on_state(data) {
        this.log('[STATE] players: ' + JSON.stringify(data));
    }
};
