// Cloudflare Workers (static assets) デプロイ用のステージング。
// kseo.ink/works/sql-flow* のルートで配信されるため、アセットは
// works/sql-flow/ プレフィックス付きのディレクトリ構造に置く。
import { cpSync, mkdirSync, rmSync } from 'node:fs'

rmSync('.deploy', { recursive: true, force: true })
mkdirSync('.deploy/works', { recursive: true })
cpSync('dist', '.deploy/works/sql-flow', { recursive: true })
console.log('staged dist -> .deploy/works/sql-flow')
