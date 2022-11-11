var ws = undefined;

const textarea = document.getElementById('console_output');
const roomid = document.getElementById('roomid');
const username = document.getElementById('username');
const data = document.getElementById('data');

function log(logdata) {
    textarea.value += logdata;
    textarea.scrollTop = textarea.scrollHeight;
}

function start_connection() {
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
        ws = new WebSocket("ws://127.0.0.1:8081", []);
        ws.onclose = (a) => { console.log(a); log("[CLOSED] Code: " + a.code + " Reason: \"" + a.reason + "\"\n"); }
        ws.onerror = (a) => { console.log(a); log("[ERROR]\n"); }
        ws.onopen = (a) => { console.log(a); log("[OPENED]\n"); }
        ws.onconnect = (a) => { console.log(a); log("[CONNECTED] " + a + "\n"); };
        ws.onmessage = (a) => { console.log(a); log("[MESSAGE] " + a + "\n"); };
    }
    else {
        log("[CONNECTING] The websocket is already connecting.\n");
    }
}

function send_message() {
    if (ws != undefined && ws.readyState == 1) {
        log("[MESSAGE OUT] Data send: " + data.value + "\n");
        ws.send(data.value);
    } else {
        log("[MESSAGE OUT] The websocket is not (Yet?) connected.\n");
    }
}
