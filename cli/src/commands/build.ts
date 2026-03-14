import chalk from 'chalk'
import ora from 'ora'
import execa from 'execa'
import fs from 'fs'
import path from 'path'
import { isNextJsProject, getAppName, getRuntimeBinary } from '../utils/nextjs'

interface BuildOptions {
  target: string
}

export async function build(options: BuildOptions) {
  const cwd = process.cwd()

  if (!isNextJsProject(cwd)) {
    console.error(chalk.red('✗ Not a Next.js project. Make sure next.config.js exists.'))
    process.exit(1)
  }

  const appName = getAppName(cwd)
  console.log(chalk.bold(`\n📦 Building ${appName} for desktop...\n`))

  // Step 1: Build Next.js
  const nextSpinner = ora('Building Next.js app...').start()
  try {
    await execa('npm', ['run', 'build'], { cwd, stdio: 'pipe' })
    nextSpinner.succeed('Next.js app built')
  } catch (err: any) {
    nextSpinner.fail(`Next.js build failed: ${err.message}`)
    process.exit(1)
  }

  // Step 2: Copy runtime binary
  const bundleSpinner = ora('Bundling desktop runtime...').start()
  try {
    const outputDir = path.join(cwd, 'dist-desktop')
    fs.mkdirSync(outputDir, { recursive: true })

    const runtime = getRuntimeBinary()
    const outputBinary = path.join(outputDir, appName)
    fs.copyFileSync(runtime, outputBinary)
    fs.chmodSync(outputBinary, 0o755)

    // Copy Next.js build output
    const nextOut = path.join(cwd, '.next')
    fs.cpSync(nextOut, path.join(outputDir, '.next'), { recursive: true })

    bundleSpinner.succeed('Runtime bundled')

    console.log(chalk.bold('\n✨ Build complete!\n'))
    console.log(chalk.gray(`  Output: ${chalk.cyan(outputDir)}`))
    console.log(chalk.gray(`  Binary: ${chalk.cyan(outputBinary)}\n`))

  } catch (err: any) {
    bundleSpinner.fail(`Bundling failed: ${err.message}`)
    process.exit(1)
  }
}
