import { Env, task } from "@terra-money/terrain";
import Lib from '../lib';

// run this using following cmd:  terrain task:run with-lib --signer pisco --network testnet 

task(async (env: Env) => {
  const lib = new Lib(env);
  console.log("Creating New Thread");
  await lib.createThread({title: "Welcome to Tefi DAgora", category: "General", content: "A decentralized agora on Terra Blockchain"})
  console.log("Thread with ID=1 \n", await lib.getThreadById({id: 1}));
});