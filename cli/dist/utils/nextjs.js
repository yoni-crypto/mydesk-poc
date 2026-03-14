"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.isNextJsProject = isNextJsProject;
exports.getPackageJson = getPackageJson;
exports.getAppName = getAppName;
exports.waitForServer = waitForServer;
exports.getRuntimeBinary = getRuntimeBinary;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const http_1 = __importDefault(require("http"));
function isNextJsProject(cwd = process.cwd()) {
    return (fs_1.default.existsSync(path_1.default.join(cwd, 'next.config.js')) ||
        fs_1.default.existsSync(path_1.default.join(cwd, 'next.config.ts')) ||
        fs_1.default.existsSync(path_1.default.join(cwd, 'next.config.mjs')));
}
function getPackageJson(cwd = process.cwd()) {
    const pkgPath = path_1.default.join(cwd, 'package.json');
    if (!fs_1.default.existsSync(pkgPath))
        return null;
    return JSON.parse(fs_1.default.readFileSync(pkgPath, 'utf-8'));
}
function getAppName(cwd = process.cwd()) {
    const pkg = getPackageJson(cwd);
    return pkg?.name ?? path_1.default.basename(cwd);
}
function waitForServer(url, timeout = 60000) {
    return new Promise((resolve, reject) => {
        const start = Date.now();
        const check = () => {
            http_1.default.get(url, (res) => {
                if (res.statusCode && res.statusCode < 500) {
                    resolve();
                }
                else {
                    retry();
                }
            }).on('error', retry);
        };
        const retry = () => {
            if (Date.now() - start > timeout) {
                reject(new Error(`Server at ${url} did not start within ${timeout}ms`));
                return;
            }
            setTimeout(check, 500);
        };
        check();
    });
}
function getRuntimeBinary() {
    // In dev: use the compiled binary from the rust project
    const devBinary = path_1.default.join(__dirname, '../../target/debug/mydesk-poc');
    if (fs_1.default.existsSync(devBinary))
        return devBinary;
    // In production: binary is bundled next to the CLI
    const prodBinary = path_1.default.join(__dirname, '../bin/mydesk-runtime');
    if (fs_1.default.existsSync(prodBinary))
        return prodBinary;
    throw new Error('MyDesk runtime binary not found. Run `cargo build` first.');
}
