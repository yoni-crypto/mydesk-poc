import fs from 'fs'
import path from 'path'
import http from 'http'

export function isNextJsProject(cwd: string = process.cwd()): boolean {
  return (
    fs.existsSync(path.join(cwd, 'next.config.js')) ||
    fs.existsSync(path.join(cwd, 'next.config.ts')) ||
    fs.existsSync(path.join(cwd, 'next.config.mjs'))
  )
}

export function getPackageJson(cwd: string = process.cwd()): Record<string, any> | null {
  const pkgPath = path.join(cwd, 'package.json')
  if (!fs.existsSync(pkgPath)) return null
  return JSON.parse(fs.readFileSync(pkgPath, 'utf-8'))
}

export function getAppName(cwd: string = process.cwd()): string {
  const pkg = getPackageJson(cwd)
  return pkg?.name ?? path.basename(cwd)
}

export function waitForServer(url: string, timeout = 60000): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now()

    const check = () => {
      http.get(url, (res) => {
        if (res.statusCode && res.statusCode < 500) {
          resolve()
        } else {
          retry()
        }
      }).on('error', retry)
    }

    const retry = () => {
      if (Date.now() - start > timeout) {
        reject(new Error(`Server at ${url} did not start within ${timeout}ms`))
        return
      }
      setTimeout(check, 500)
    }

    check()
  })
}

export function getRuntimeBinary(): string {
  // In dev: use the compiled binary from the rust project
  const devBinary = path.join(__dirname, '../../../target/debug/mydesk-poc')
  if (fs.existsSync(devBinary)) return devBinary

  // In production: binary is bundled next to the CLI
  const prodBinary = path.join(__dirname, '../bin/mydesk-runtime')
  if (fs.existsSync(prodBinary)) return prodBinary

  throw new Error('MyDesk runtime binary not found. Run `cargo build` first.')
}
