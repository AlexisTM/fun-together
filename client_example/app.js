var app;
window.onload = () => {
    const { createApp } = Vue;

    app = createApp({
        data() {
            return {
                start_tab: 'join', // The start tab on login mode ['join', 'create']
                mode: 'login', // The main page. ['login', 'host', 'player']
                game: new Game(),
                room_id_player: "",
                data_input_player: "{'some': 'data'}",
                data_input_host: "{'some': 'data'}",
                data_to_host: "[]",
                room_id_host: "",
                max_players: 8,
                accept_players: false,
                players: [],
            }
        },
        methods: {
            join() {
                if (this.game.join(this.room_id_player.toUpperCase())) {
                    this.mode = 'player';
                }
            },
            create() {
                if (this.game.create()) {
                    this.mode = 'host';
                }
            },
            stop() {
                this.game.stop();
                this.mode = 'login';
                this.room_id_host = "";
            }
        },
        mounted() {
            this.game.on_prepare_reply = (data) => {
                this.room_id_host = data.key;
            };
            this.game.on_player_data = (data) => {
                console.log(data);
            };
            this.game.on_stop = (data) => {
                console.log(data);
            };
            this.game.on_error = (data) => {
                console.log(data);
            };
            this.game.on_state = (data) => {
                this.accept_players = data.accept_conns;
                this.players = data.players;
                console.log(data);
            };
            this.game.on_host_str = (data) => {
                console.log("Str data: ", data);
            }
            this.game.on_host_bin = (data) => {
                console.log("Bin data: ", CBOR.decode(data), " from message:", data);
            }
            this.game.on_ws_close = (data) => {
                console.log(data);
                this.mode = 'login'
            }
        },
    }).mount('#app')
}
