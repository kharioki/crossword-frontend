import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';

import getConfig from './config';
import { viewMethodOnContract } from './utils';
import { data } from './hardcoded-data';

// initCrossword called before passing data to the app.
// pass env variable NEAR_ENV used to designate the blockchain network (testnet, betanet, mainnet)
async function initCrossword() {
  const nearConfig = getConfig(process.env.NEAR_ENV || 'testnet');
  const solutionHash = await viewMethodOnContract(nearConfig, 'get_solution');
  return { data, solutionHash }
}

initCrossword()
  .then(({ data, solutionHash }) => {
    ReactDOM.render(
      <React.StrictMode>
        <App data={data} solutionHash={solutionHash} />
      </React.StrictMode>,
      document.getElementById('root')
    );
  });
