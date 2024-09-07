import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";

chai.use(chaiAsPromised);
process.on('unhandledRejection', (err) => {
    // console.error(err);
    // process.exit(1);
})

/** define test suites here **/

import { initialize } from "./restaking/1_initialize";
import { deposit_sol } from "./restaking/2_deposit_sol";
// import { deposit_token } from "./restaking/3_deposit_token";
// import { transfer_hook } from "./restaking/4_transfer_hook";
// import { withdraw } from "./restaking/5_withdraw";
// import { reward } from "./restaking/6_reward";

initialize;
deposit_sol;
// deposit_token;
// transfer_hook;
// withdraw;
// reward;