import { readFileSync, writeFileSync } from 'fs';
import glob from 'glob';
import { resolve } from 'path';

const syncYMLPath = '.github/sync.yml';
let syncYML = readFileSync(resolve(syncYMLPath)).toString();
const syncFile = glob.sync('.dfx/local/canisters/**/*.{ts,did}', {
  cwd: resolve('./'),
});

function getTemplate(source, dest) {
  const template = `  - source: ${source}
    dest: ${dest}
    replace: true
    deleteOrphaned: true\n`;
  return template;
}

for (const file of syncFile) {
  const base = file.match(/canisters([/\w.]+)$/)[1];
  const dest = `src${base}`;
  syncYML += getTemplate(file, dest);
}
writeFileSync(resolve(syncYMLPath), syncYML);

console.log('syncYML', syncYML);
console.log('syncFile', syncFile);
