#!/usr/bin/env node
import { program } from 'commander'
import { dev } from './commands/dev'
import { build } from './commands/build'

program
  .name('mydesk')
  .description('Turn your Next.js app into a desktop app in 30 seconds')
  .version('0.1.0')

program
  .command('dev')
  .description('Start your Next.js app as a desktop app')
  .option('-p, --port <port>', 'Next.js dev server port', '3000')
  .action(dev)

program
  .command('build')
  .description('Build your app for production')
  .option('-t, --target <target>', 'Target platform (windows, macos, linux)', process.platform)
  .action(build)

program.parse()
