import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve, join } from 'path'
import { createReadStream, existsSync, statSync, readdirSync } from 'fs'

export default defineConfig({
  plugins: [
    react(),
    {
      name: 'serve-logs',
      configureServer(server) {
        const logsDir = resolve(__dirname, '../../logs')
        server.middlewares.use('/api/data', (req, res, _next) => {
          const urlPath = (req.url ?? '/').split('?')[0].replace(/^\//, '')
          const filePath = join(logsDir, urlPath)

          // Prevent path traversal
          if (!filePath.startsWith(logsDir)) {
            res.statusCode = 403
            res.end('Forbidden')
            return
          }

          if (!existsSync(filePath)) {
            res.statusCode = 404
            res.end('Not found')
            return
          }

          const stat = statSync(filePath)
          if (stat.isDirectory()) {
            res.setHeader('Content-Type', 'application/json')
            res.setHeader('Access-Control-Allow-Origin', '*')
            res.end(JSON.stringify(readdirSync(filePath)))
            return
          }

          const ext = filePath.split('.').pop() ?? ''
          const mime: Record<string, string> = {
            json: 'application/json',
            jsonl: 'text/plain',
            log: 'text/plain',
          }
          res.setHeader('Content-Type', mime[ext] ?? 'application/octet-stream')
          res.setHeader('Access-Control-Allow-Origin', '*')
          createReadStream(filePath).pipe(res)
        })
      },
    },
  ],
  server: {
    port: 5174,
    strictPort: true,
  },
})
