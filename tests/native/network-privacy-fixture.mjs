import { createServer as createHttpServer } from 'node:http';
import { createServer as createTcpServer } from 'node:net';

// Loopback-only fixtures. The SOCKS endpoint records requests and always
// refuses them; it never forwards traffic or acts as a Tor implementation.
export async function createNetworkPrivacyFixture() {
  const httpRequests = [];
  const socksRequests = [];
  const sockets = new Set();
  const http = createHttpServer((request, response) => {
    httpRequests.push(request.url);
    if (request.url === '/drop') { request.socket.destroy(); return; }
    if (request.url === '/redirect-http') { response.writeHead(302, { location: '/page' }).end(); return; }
    response.writeHead(200, { 'content-type': 'text/plain', 'cache-control': 'no-store' }).end('fixture');
  });
  const socks = createTcpServer(socket => {
    sockets.add(socket);
    socket.on('close', () => sockets.delete(socket));
    socket.on('error', () => {});
    let buffer = Buffer.alloc(0);
    let greeting = true;
    socket.on('data', chunk => {
      buffer = Buffer.concat([buffer, chunk]);
      if (greeting) {
        if (buffer.length < 2 || buffer.length < 2 + buffer[1]) return;
        if (buffer[0] !== 5 || !buffer.subarray(2, 2 + buffer[1]).includes(0)) { socket.destroy(); return; }
        buffer = buffer.subarray(2 + buffer[1]);
        greeting = false;
        socket.write(Buffer.from([5, 0]));
      }
      if (buffer.length < 5) return;
      if (buffer[0] !== 5 || buffer[3] !== 3) { socket.destroy(); return; }
      const length = buffer[4];
      if (buffer.length < 7 + length) return;
      socksRequests.push({ command: buffer[1], host: buffer.subarray(5, 5 + length).toString('ascii'), port: buffer.readUInt16BE(5 + length) });
      socket.end(Buffer.from([5, 1, 0, 1, 0, 0, 0, 0, 0, 0]));
    });
  });
  const listen = server => new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => { server.off('error', reject); resolve(server.address().port); });
  });
  const httpPort = await listen(http);
  let socksPort;
  try { socksPort = await listen(socks); } catch (error) { await new Promise(resolve => http.close(resolve)); throw error; }
  return {
    origin: `http://127.0.0.1:${httpPort}`,
    socksPort,
    httpRequests,
    socksRequests,
    async close() {
      for (const socket of sockets) socket.destroy();
      http.closeAllConnections();
      await Promise.all([new Promise(resolve => http.close(resolve)), new Promise(resolve => socks.close(resolve))]);
    },
  };
}
