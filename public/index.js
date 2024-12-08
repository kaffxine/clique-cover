document.addEventListener('DOMContentLoaded', () => {
    console.log("hello from javascript :3");
    const wsProtocol = window.location.protocol === 'https:' ? 'https:' : 'http:';
    const wsHost = window.location.host
    const wsPath = 'ws';
    // const wsUrl = `${wsProtocol}//${wsHost}/${wsPath}`;
    const wsUrl = 'http://localhost:8080/ws'
    const socket = new WebSocket(wsUrl);
    console.log("websocket created");

    socket.addEventListener('open', () => {
        console.log('ws connection established');
    });

    socket.addEventListener('error', (error) => {
        console.error('ws error', error);
    });

    let launchButton = document.getElementById('button-launch');

    launchButton.addEventListener('click', () => {
        console.log('socket state:', socket.readyState);
        if (socket.readyState === WebSocket.OPEN) {
            let sample = { uwu: 'owo' };
            console.log("attempting to send:", sample);
            socket.send(JSON.stringify(sample));
        }
    });

});



