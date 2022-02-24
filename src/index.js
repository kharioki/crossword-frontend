import React from 'react';
import ReactDOM from 'react-dom';
import * as nearAPI from 'near-api-js';
import App from './App';
import getConfig from './config.js';
import { viewMethodOnContract, mungeBlockchainCrossword } from './utils';

async function initCrossword() {
  const nearConfig = getConfig(process.env.NEAR_ENV || 'testnet');
  // console.log('nearConfig', nearConfig);

  // create a keyStore for signing transactions using the user's key
  // which is located in the browser local storage after user logs in
  const keyStore = new nearAPI.keyStores.BrowserLocalStorageKeyStore();
  // console.log({ keyStore });

  // Initializing connection to the NEAR testnet
  const near = await nearAPI.connect({ keyStore, ...nearConfig });
  // console.log({ near });

  // Initialize wallet connection
  const walletConnection = new nearAPI.WalletConnection(near);
  // console.log({ walletConnection });

  // let currentUser = walletConnection.getAccountId();
  // console.log({ currentUser });
  // const walletAccountObj = walletConnection.account();
  // console.log({ walletAccountObj });

  // Load in user's account data
  let currentUser;
  if (walletConnection.getAccountId()) {
    currentUser = await walletConnection.getAccountId();
  }

  const chainData = await viewMethodOnContract(nearConfig, 'get_unsolved_puzzles', '{}');
  console.log('chainData', chainData);

  let data;
  let solutionHash;

  // There may not be any crossword puzzles to solve, check this.
  if (chainData.puzzles.length) {
    solutionHash = chainData.puzzles[0]['solution_hash'];
    data = mungeBlockchainCrossword(chainData.puzzles);
    console.log({ data });
  } else {
    console.log("Oof, there's no crossword to play right now, friend.");
  }
  return { data, solutionHash, nearConfig, walletConnection, currentUser };
}

initCrossword()
  .then(({ data, solutionHash, nearConfig, walletConnection, currentUser }) => {
    console.log({ data, solutionHash, nearConfig, walletConnection, currentUser });
    ReactDOM.render(
      <App
        data={data}
        hash={solutionHash}
        nearConfig={nearConfig}
        walletConnection={walletConnection}
        currentUser={currentUser}
      />,
      document.getElementById('root'));
  });