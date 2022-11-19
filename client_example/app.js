var app;
window.onload = () => {
    const { createApp } = Vue;

    app = createApp({
        data() {
            return {
                start_tab: 'join',
                mode: 'login', // ['login', 'host', 'player']
                game: new Game(),
                room_id_player: "",
                data_input_player: "{'some': 'data'}",
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
                    console.log('Join ok')
                } else {
                    console.log('Join failed')
                }
            },
            create() {
                if (this.game.create()) {
                    this.mode = 'host';
                }
            },
            stop() {
                this.game.stop();
                this.mode = 'login'
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
            };
            this.game.on_host_data = (data) => {
                console.log("host data: " + data);
            }
        },
    }).mount('#app')
}
