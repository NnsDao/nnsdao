import { readFileSync, writeFileSync } from 'fs';
import glob from 'glob';
import { resolve } from 'path';

const dfxJson = JSON.parse(readFileSync(relativeToRootPath('dfx.json')).toString());
const syncYMLPath = '.github/sync.yml';
let syncYML = readFileSync(resolve(syncYMLPath)).toString();
const syncFile = glob.sync('.dfx/ic/canisters/**/*.{ts,did}', {
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

  // filter out some
  const canisterName = base.match(/\w+/)[0];
  const candidPath = dfxJson.canisters?.[canisterName]?.candid;
  if (!candidPath) continue;

  const dest = `src${base}`;
  syncYML += getTemplate(file, dest);
}
writeFileSync(resolve(syncYMLPath), syncYML);

console.log('syncYML', syncYML);
console.log('syncFile', syncFile);

function relativeToRootPath(url) {
  return resolve(process.cwd(), url);
}
