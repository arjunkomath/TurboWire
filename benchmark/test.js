import { check, sleep } from 'k6';
import ws from 'k6/ws';

export default function () {
  const url = __ENV.WS_URL || 'ws://localhost:8080';

  const res = ws.connect(url, null, function (socket) {
    socket.on('open', function () {
      console.log('Connected to WebSocket server');

      // Send a message to the server
      const message = JSON.stringify({ action: 'greet', data: 'Hello, server!' });
      socket.send(message);
      console.log(`Sent message: ${message}`);
    });

    socket.on('message', function (message) {
      console.log(`Received message: ${message}`);
      check(message, { 'is message valid': (msg) => msg !== '' });
    });

    socket.setTimeout(function () {
      console.log('Closing socket');
      socket.close();
    }, 5000); // Close the socket after 5 seconds
  });

  check(res, { 'status is 101': (r) => r && r.status === 101 });
  sleep(1); // Sleep for 1 second before the next iteration
}


export let options = {
  vus: 500,
  duration: '30s',
};

