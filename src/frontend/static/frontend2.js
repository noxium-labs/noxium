import init, { greet } from '/pkg/hello.js';

async function run() {
    await init();

    document.getElementById('greet').addEventListener('click', () => {
        const message = greet('Frontend 2 User');
        document.getElementById('message').innerText = message;
    });
}

run();