import chalk from 'chalk'
import ora from 'ora'
import { execa } from 'execa'
import { isNextJsProject, waitForServer, getRuntimeBinary, getAppName } from '../utils/nextjs'

interface DevOptions {
  port: string
}

export async function dev(options: DevOptions) {
  const port = options.port
  const url = `http://localhost:${port}`
  const cwd = process.cwd()

  // Check if Next.js project
  if (!isNextJsProject(cwd)) {
    console.error(chalk.red('✗ Not a Next.js project. Make sure next.config.js exists.'))
    process.exit(1)
  }

  const appName = getAppName(cwd)
  console.log(chalk.bold(`\n🚀 MyDesk — ${appName}\n`))

  // Start Next.js dev server
  const nextSpinner = ora('Starting Next.js dev server...').start()

  const nextProcess = execa('npm', ['run', 'dev'], {
    cwd,
    env: { ...process.env, PORT: port },
    stdio: ['ignore', 'pipe', 'pipe'],
  })

  nextProcess.stdout?.on('data', (data: Buffer) => {
    const line = data.toString()
    if (line.includes('Ready') || line.includes('ready')) {
      nextSpinner.succeed(`Next.js ready on ${chalk.cyan(url)}`)
    }
  })

  nextProcess.stderr?.on('data', (data: Buffer) => {
    const line = data.toString()
    if (line.toLowerCase().includes('error')) {
      console.error(chalk.red(line.trim()))
    }
  })

  // Wait for server to be ready
  try {
    await waitForServer(url)
  } catch (err) {
    nextSpinner.fail('Next.js dev server failed to start')
    nextProcess.kill()
    process.exit(1)
  }

  // Launch desktop window
  const windowSpinner = ora('Opening desktop window...').start()

  try {
    const runtime = getRuntimeBinary()

    const runtimeProcess = execa(runtime, [], {
      env: {
        ...process.env,
        MYDESK_URL: url,
        MYDESK_TITLE: appName,
        GDK_BACKEND: 'x11',
        WAYLAND_DISPLAY: '',
      },
      stdio: 'inherit',
    })

    windowSpinner.succeed(`${chalk.green('✨ Ready!')} Desktop window opened`)
    console.log(chalk.gray(`\n  App:     ${chalk.cyan(url)}`))
    console.log(chalk.gray(`  Runtime: ${runtime}\n`))

    // Clean shutdown
    const cleanup = () => {
      nextProcess.kill()
      runtimeProcess.kill()
      process.exit(0)
    }

    process.on('SIGINT', cleanup)
    process.on('SIGTERM', cleanup)

    await runtimeProcess

  } catch (err: any) {
    windowSpinner.fail(`Failed to open desktop window: ${err.message}`)
    nextProcess.kill()
    process.exit(1)
  }
}
