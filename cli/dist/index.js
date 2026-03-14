#!/usr/bin/env node
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const commander_1 = require("commander");
const dev_1 = require("./commands/dev");
const build_1 = require("./commands/build");
commander_1.program
    .name('mydesk')
    .description('Turn your Next.js app into a desktop app in 30 seconds')
    .version('0.1.0');
commander_1.program
    .command('dev')
    .description('Start your Next.js app as a desktop app')
    .option('-p, --port <port>', 'Next.js dev server port', '3000')
    .action(dev_1.dev);
commander_1.program
    .command('build')
    .description('Build your app for production')
    .option('-t, --target <target>', 'Target platform (windows, macos, linux)', process.platform)
    .action(build_1.build);
commander_1.program.parse();
