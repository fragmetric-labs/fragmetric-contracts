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

if (process.env.JUST_OPERATE) {
    require('./restaking/2_deposit_sol')(1);
    require('./restaking/7_operate_todo')(1);

} else if (process.env.JUST_STAKE) {
    require('./restaking/2_deposit_sol')(1);
    require('./restaking/8_operate_deprecating')(1);
    require('./restaking/8_operator_deprecating_spl_stake_pool');

} else if (!process.env.JUST_INIT) {
    require('./restaking/2_deposit_sol')(1);
    require('./restaking/3_deposit_token')(1);
    require('./restaking/4_withdraw');
    require('./restaking/5_transfer_hook');
    require('./restaking/6_reward');
    require('./restaking/8_operate_deprecating')(1);
    require('./restaking/2_deposit_sol')(2);
    require('./restaking/3_deposit_token')(2);
    require('./restaking/8_operate_deprecating')(2);
}
