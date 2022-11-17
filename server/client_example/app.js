var ws = undefined;

const textarea = document.getElementById('console_output');
const roomid = document.getElementById('roomid');
const username = document.getElementById('username');
const data = document.getElementById('data');

var CMD_PREPARE = {

};

var msgs = []

function log(logdata) {
    textarea.value += logdata;
    textarea.scrollTop = textarea.scrollHeight;
}

function connect_game() {
    start_connection("CONNECT");
}

function create_game() {
    start_connection("CREATE");
}

function start_connection(type) {
    // location.href
    if (ws != undefined && (ws.readyState == 2 || ws.readyState == 3)) {
        ws.onclose = undefined;
        ws.onerror = undefined;
        ws.onopen = undefined;
        ws.onconnect = undefined;
        ws.onmessage = undefined;
        ws.close();
        ws = undefined;
    }
    if (ws == undefined) {
        log("[CONNECTING] to " + roomid.value + " as " + username.value + " \n");
        ws = new WebSocket("ws://127.0.0.1:8081/" + type + "/" + roomid.value);
        ws.binaryType = "arraybuffer";
        ws.onclose = (a) => { console.log(a); log("[CLOSED] Code: " + a.code + " Reason: \"" + a.reason + "\"\n"); }
        ws.onerror = (a) => { console.log(a); log("[ERROR]\n"); }
        ws.onopen = (a) => { console.log(a); log("[OPENED]\n"); }
        ws.onconnect = (a) => { console.log(a); log("[CONNECTED]\n"); };
        ws.onmessage = (a) => {
            msgs = a;
            log("[MESSAGE   IN] " + JSON.stringify(CBOR.decode(a.data)) + "\n");
            console.log(a);
        };
    }
    else {
        log("[CONNECTING] The websocket is already connecting.\n");
    }
}

function send_message() {
    if (ws != undefined && ws.readyState == 1) {
        log("[MESSAGE OUT] Data send: " + data.value + "\n");
        ws.send_binary(CBOR.encode(JSON.parse(data.value)));
    } else {
        log("[MESSAGE OUT] The websocket is not (Yet?) connected.\n");
    }
}
