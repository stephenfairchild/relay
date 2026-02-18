const express = require('express');
const app = express();
const port = 3000;

// Simple request counter to demonstrate caching
let requestCount = 0;

// Health check endpoint
app.get('/health', (req, res) => {
        res.json({ status: 'ok' });
});

// Simple API endpoint
app.get('/api/data', (req, res) => {
        requestCount++;
        res.json({
                message: 'Hello from Express!',
                timestamp: new Date().toISOString(),
                requestCount: requestCount,
                note: 'This response is cached by Relay for 30s'
        });
});

// Simulate a slow endpoint
app.get('/api/slow', (req, res) => {
        requestCount++;
        setTimeout(() => {
                res.json({
                        message: 'This endpoint takes 2 seconds to respond',
                        timestamp: new Date().toISOString(),
                        requestCount: requestCount,
                        note: 'This response is cached by Relay for 2m'
                });
        }, 2000);
});

// Static content example
app.get('/static/info', (req, res) => {
        requestCount++;
        res.json({
                message: 'Static content',
                timestamp: new Date().toISOString(),
                requestCount: requestCount,
                note: 'This response is cached by Relay for 1 day'
        });
});

// Endpoint with custom cache headers
app.get('/api/custom-cache', (req, res) => {
        requestCount++;
        res.set('Cache-Control', 'public, max-age=300');
        res.json({
                message: 'Custom cache headers',
                timestamp: new Date().toISOString(),
                requestCount: requestCount,
                note: 'This uses Cache-Control header to set TTL'
        });
});

// No-cache endpoint
app.get('/api/no-cache', (req, res) => {
        requestCount++;
        res.set('Cache-Control', 'no-store');
        res.json({
                message: 'This should not be cached',
                timestamp: new Date().toISOString(),
                requestCount: requestCount
        });
});

app.listen(port, () => {
        console.log(`Express backend listening on port ${port}`);
        console.log(`Health check available at http://localhost:${port}/health`);
});
