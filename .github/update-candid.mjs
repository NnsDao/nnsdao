import { Actor, HttpAgent } from '@dfinity/agent';
import { readFileSync, writeFileSync } from 'fs';
import glob from 'glob';
import { resolve } from 'path';

const idlFactory = ({ IDL }) =>
  IDL.Service({
    __get_candid_interface_tmp_hack: IDL.Func([], [IDL.Text], ['query']),
  });

const onlineHost = 'https://ic0.app';
const anonymousAgent = new HttpAgent({ host: onlineHost });

const canister_ids = JSON.parse(readFileSync(relativeToRootPath('canister_ids.json')).toString());

Object.keys(canister_ids).forEach(async canisterName => {
  const cid = canister_ids[canisterName].ic;
  const candidStr = await getCandid(cid);
  const isRustProject = glob.sync('*.toml', { cwd: resolve('./') });
  if (isRustProject.length) {
    try {
      // @ts-ignore
      writeFileSync(relativeToRootPath(`./src/${canisterName}/${canisterName}.did`), candidStr);
    } catch (error) {
      // if canister_ids config multiple canisterId,such as test,prod,xxx,ignore this error
      console.error('error', error);
    }
  }
});

function relativeToRootPath(url) {
  return resolve(process.cwd(), url);
}

async function getCandid(cid) {
  const actor = Actor.createActor(idlFactory, {
    agent: anonymousAgent,
    canisterId: cid,
  });
  return await actor.__get_candid_interface_tmp_hack();
}
