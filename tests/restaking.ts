import * as chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import {RestakingPlayground} from "../tools/restaking/playground";
import * as anchor from "@coral-xyz/anchor";

export const restakingPlayground = RestakingPlayground.create('local', {
    provider: anchor.AnchorProvider.env(),
});

chai.use(chaiAsPromised);
process.on('unhandledRejection', (err) => {
    console.error(err);
    process.exit(1);
});

/** define test suites here **/

require('./restaking/1_initialize');

if (!process.env.JUST_INIT) {
    require('./restaking/2_deposit_sol');
    require('./restaking/3_deposit_token');
    require('./restaking/4_withdraw');
    require('./restaking/5_transfer_hook');
    require('./restaking/6_reward');
    require('./restaking/7_operator_spl_stake_pool');
}
