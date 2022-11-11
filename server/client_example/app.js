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
    if (ws == undefined) {
        log("[CONNECTING] to " + roomid.value + " as " + username.value + " \n");
        ws = new WebSocket("ws://127.0.0.1:8081", []);
        ws.onclose = (a) => { console.log(a); log("[CLOSED] Code: " + a.code + " Reason: \"" + a.reason + "\"\n");}
        ws.onerror = (a) => { console.log(a); log("[ERROR]\n"); }
        ws.onopen = (a) => { console.log(a); log("[OPENED]\n"); }
        ws.onconnect = (a) => { console.log(a); log("[CONNECTED] " + a + "\n"); };
        ws.onmessage = (a) => { console.log(a); log("[MESSAGE] " + a + "\n"); };
    }
    else {
        log("[CONNECTING] The websocket is already connecting.\n");
    }
}

function send_data() {
    if (ws != undefined) {
        log("[DATA] Data send: " + data.value + "\n");
        ws.send(data.value);
    } else {
        log("[DATA] The websocket is not connected.\n");
    }
}
