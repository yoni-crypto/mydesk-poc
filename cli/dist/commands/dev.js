"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.dev = dev;
const chalk_1 = __importDefault(require("chalk"));
const ora_1 = __importDefault(require("ora"));
const execa_1 = require("execa");
const nextjs_1 = require("../utils/nextjs");
async function dev(options) {
    const port = options.port;
    const url = `http://localhost:${port}`;
    const cwd = process.cwd();
    // Check if Next.js project
    if (!(0, nextjs_1.isNextJsProject)(cwd)) {
        console.error(chalk_1.default.red('✗ Not a Next.js project. Make sure next.config.js exists.'));
        process.exit(1);
    }
    const appName = (0, nextjs_1.getAppName)(cwd);
    console.log(chalk_1.default.bold(`\n🚀 MyDesk — ${appName}\n`));
    // Start Next.js dev server
    const nextSpinner = (0, ora_1.default)('Starting Next.js dev server...').start();
    const nextProcess = (0, execa_1.execa)('npm', ['run', 'dev'], {
        cwd,
        env: { ...process.env, PORT: port },
        stdio: ['ignore', 'pipe', 'pipe'],
    });
    nextProcess.stdout?.on('data', (data) => {
        const line = data.toString();
        if (line.includes('Ready') || line.includes('ready')) {
            nextSpinner.succeed(`Next.js ready on ${chalk_1.default.cyan(url)}`);
        }
    });
    nextProcess.stderr?.on('data', (data) => {
        const line = data.toString();
        if (line.toLowerCase().includes('error')) {
            console.error(chalk_1.default.red(line.trim()));
        }
    });
    // Wait for server to be ready
    try {
        await (0, nextjs_1.waitForServer)(url);
    }
    catch (err) {
        nextSpinner.fail('Next.js dev server failed to start');
        nextProcess.kill();
        process.exit(1);
    }
    // Launch desktop window
    const windowSpinner = (0, ora_1.default)('Opening desktop window...').start();
    try {
        const runtime = (0, nextjs_1.getRuntimeBinary)();
        const runtimeProcess = (0, execa_1.execa)(runtime, [], {
            env: {
                ...process.env,
                MYDESK_URL: url,
                MYDESK_TITLE: appName,
                GDK_BACKEND: 'x11',
                WAYLAND_DISPLAY: '',
            },
            stdio: 'inherit',
        });
        windowSpinner.succeed(`${chalk_1.default.green('✨ Ready!')} Desktop window opened`);
        console.log(chalk_1.default.gray(`\n  App:     ${chalk_1.default.cyan(url)}`));
        console.log(chalk_1.default.gray(`  Runtime: ${runtime}\n`));
        // Clean shutdown
        const cleanup = () => {
            nextProcess.kill();
            runtimeProcess.kill();
            process.exit(0);
        };
        process.on('SIGINT', cleanup);
        process.on('SIGTERM', cleanup);
        await runtimeProcess;
    }
    catch (err) {
        windowSpinner.fail(`Failed to open desktop window: ${err.message}`);
        nextProcess.kill();
        process.exit(1);
    }
}
