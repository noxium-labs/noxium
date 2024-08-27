import init, { greet } from './crate';

async function run() {
    await init();
    console.log(greet('World'));
}

run();