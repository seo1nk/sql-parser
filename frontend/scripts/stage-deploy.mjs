// Cloudflare Workers (static assets) デプロイ用のステージング。
// kseo.ink/works/sql-visualizer* のルートで配信されるため、アセットは
// works/sql-visualizer/ プレフィックス付きのディレクトリ構造に置く。
import { cpSync, mkdirSync, rmSync } from 'node:fs'

rmSync('.deploy', { recursive: true, force: true })
mkdirSync('.deploy/works', { recursive: true })
cpSync('dist', '.deploy/works/sql-visualizer', { recursive: true })
console.log('staged dist -> .deploy/works/sql-visualizer')
