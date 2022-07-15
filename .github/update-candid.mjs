import { Actor, HttpAgent } from '@dfinity/agent';
import { readFileSync, writeFileSync } from 'fs';
import { resolve } from 'path';

const idlFactory = ({ IDL }) =>
  IDL.Service({
    __get_candid_interface_tmp_hack: IDL.Func([], [IDL.Text], ['query']),
  });

const onlineHost = 'https://ic0.app';
const anonymousAgent = new HttpAgent({ host: onlineHost });

const canister_ids = JSON.parse(readFileSync(relativeToRootPath('canister_ids.json')).toString());
const dfxJson = JSON.parse(readFileSync(relativeToRootPath('dfx.json')).toString());

const canisterNameList = Object.keys(canister_ids);

// in case of candid point to same file,sync
for (let canisterName of canisterNameList) {
  //   if provide corresponded test canister, use test
  const cid =
    canister_ids[`${canisterName}_test`]?.ic ||
    canister_ids[`${canisterName}-test`]?.ic ||
    canister_ids[canisterName].ic;
  const candidPath = dfxJson.canisters?.[canisterName]?.candid;

  if (!candidPath) continue;

  try {
    const candidStr = await getCandid(cid);
    if (!candidStr) {
      console.warn('candid Not found');
      process.exit(1);
    }
    console.log('Found latest candid', candidStr);
    // @ts-ignore
    writeFileSync(relativeToRootPath(`${candidPath}`), candidStr);
  } catch (error) {
    // if canister_ids config multiple canisterId,such as test,prod,xxx,ignore this error
    console.error('error', error);
  }
}

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
