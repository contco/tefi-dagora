import type { Env } from "@terra-money/terrain";
import { TefiDagoraClient } from './clients/TefiDagoraClient';
import terraRef from '../refs.terrain.json'
export class Lib extends TefiDagoraClient {
  env: Env;

  constructor(env: Env) {
    super(env.client, env.defaultWallet, env.refs['tefi_dagora'].contractAddresses.default);
    this.env = env;
  }
};

export default Lib;
