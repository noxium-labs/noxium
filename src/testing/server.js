const http = require('http');
const fs = require('fs');
const path = require('path');

const server = http.createServer((req, res) => {
    const filePath = path.join(__dirname, req.url === '/' ? 'index.html' : req.url);
    fs.readFile(filePath, (err, content) => {
        if (err) {
            res.writeHead(404);
            res.end("File not found");
            return;
        }
        res.writeHead(200, { 'Content-Type': getContentType(filePath) });
        res.end(content);
    });
});

const getContentType = (filePath) => {
    const ext = path.extname(filePath);
    switch (ext) {
        case '.html': return 'text/html';
        case '.js': return 'application/javascript';
        case '.css': return 'text/css';
        default: return 'text/plain';
    }
};

server.listen(8000, () => console.log('Server running on port 8000'));