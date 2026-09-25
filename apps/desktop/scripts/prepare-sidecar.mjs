import { execFileSync } from 'node:child_process';
import { mkdirSync, copyFileSync, chmodSync } from 'node:fs';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const desktop = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const root = resolve(desktop, '../..');
const rust = process.env.VESTIGE_RUST_TOOLCHAIN ? [`+${process.env.VESTIGE_RUST_TOOLCHAIN}`] : [];
execFileSync('pnpm', ['--filter', '@vestige/dashboard', 'build'], { cwd: root, stdio: 'inherit' });
const features = process.platform === 'darwin' && process.arch === 'arm64' ? ['--features', 'qwen3-embeddings,metal'] : [];
execFileSync('cargo', [...rust, 'build', '--release', '-p', 'vestige-mcp', '--bins', ...features], { cwd: root, stdio: 'inherit' });
const metadata = JSON.parse(execFileSync('cargo', [...rust, 'metadata', '--no-deps', '--format-version', '1'], { cwd: root, encoding: 'utf8' }));
const target = execFileSync('rustc', [...rust, '--print', 'host-tuple'], { cwd: root, encoding: 'utf8' }).trim();
const extension = process.platform === 'win32' ? '.exe' : '';
const destination = join(desktop, 'src-tauri', 'binaries');
mkdirSync(destination, { recursive: true });
for (const [binary, bundledName] of [['vestige-mcp', 'vestige-service'], ['vestige', 'vestige-cli'], ['vestige-restore', 'vestige-restore']]) {
  const bundled = join(destination, `${bundledName}-${target}${extension}`);
  copyFileSync(join(metadata.target_directory, 'release', `${binary}${extension}`), bundled);
  chmodSync(bundled, 0o755);
}
console.log(`Bundled native memory service for ${target}`);
