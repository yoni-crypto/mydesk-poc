"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.build = build;
const chalk_1 = __importDefault(require("chalk"));
const ora_1 = __importDefault(require("ora"));
const execa_1 = require("execa");
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const nextjs_1 = require("../utils/nextjs");
async function build(options) {
    const cwd = process.cwd();
    if (!(0, nextjs_1.isNextJsProject)(cwd)) {
        console.error(chalk_1.default.red('✗ Not a Next.js project. Make sure next.config.js exists.'));
        process.exit(1);
    }
    const appName = (0, nextjs_1.getAppName)(cwd);
    console.log(chalk_1.default.bold(`\n📦 Building ${appName} for desktop...\n`));
    // Step 1: Build Next.js
    const nextSpinner = (0, ora_1.default)('Building Next.js app...').start();
    try {
        await (0, execa_1.execa)('npm', ['run', 'build'], { cwd, stdio: 'pipe' });
        nextSpinner.succeed('Next.js app built');
    }
    catch (err) {
        nextSpinner.fail(`Next.js build failed: ${err.message}`);
        process.exit(1);
    }
    // Step 2: Copy runtime binary
    const bundleSpinner = (0, ora_1.default)('Bundling desktop runtime...').start();
    try {
        const outputDir = path_1.default.join(cwd, 'dist-desktop');
        fs_1.default.mkdirSync(outputDir, { recursive: true });
        const runtime = (0, nextjs_1.getRuntimeBinary)();
        const outputBinary = path_1.default.join(outputDir, appName);
        fs_1.default.copyFileSync(runtime, outputBinary);
        fs_1.default.chmodSync(outputBinary, 0o755);
        // Copy Next.js build output
        const nextOut = path_1.default.join(cwd, '.next');
        fs_1.default.cpSync(nextOut, path_1.default.join(outputDir, '.next'), { recursive: true });
        bundleSpinner.succeed('Runtime bundled');
        console.log(chalk_1.default.bold('\n✨ Build complete!\n'));
        console.log(chalk_1.default.gray(`  Output: ${chalk_1.default.cyan(outputDir)}`));
        console.log(chalk_1.default.gray(`  Binary: ${chalk_1.default.cyan(outputBinary)}\n`));
    }
    catch (err) {
        bundleSpinner.fail(`Bundling failed: ${err.message}`);
        process.exit(1);
    }
}
