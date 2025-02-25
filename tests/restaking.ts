import * as chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import {RestakingPlayground} from "../tools/restaking/playground";
import { RestakingPlayground as JTORestakingPlayground } from '../tools/restaking/jto_playground';
import * as anchor from "@coral-xyz/anchor";

chai.use(chaiAsPromised);
process.on('unhandledRejection', (err) => {
    console.error(err);
    process.exit(1);
});

export const restakingPlayground = (process.env.JTO)
    ? JTORestakingPlayground.create('local', {
        provider: anchor.AnchorProvider.env(),
    }) : RestakingPlayground.create('local', {
        provider: anchor.AnchorProvider.env(),
    });


/** define test suites here **/
if (process.env.JTO) {
    require('./restaking/1_initialize_jto');

    if (process.env.JUST_WITHDRAW) {
        require('./restaking/4_withdraw_jto');

    } else if (process.env.JUST_OPERATE) {
        require('./restaking/7_operate_jto')(1);

    }
} else {
    require('./restaking/1_initialize');

    if (process.env.JUST_WITHDRAW_TOKEN) {
        require('./restaking/4_withdraw_token');

    } else if (process.env.JUST_DONATE) {
        require('./restaking/2_donate_sol');
        
    } else if (process.env.JUST_OPERATE) {
        require('./restaking/7_operate')(1);

    } else if (process.env.JUST_OPERATE2) {
        require('./restaking/7_operate2')(1);

    } else if (process.env.JUST_STAKE) {
        require('./restaking/2_deposit_sol')(1);
        require('./restaking/8_operator_deprecating_spl_stake_pool');

    } else if (process.env.JUST_DENORMALIZE) {
        require('./restaking/2_deposit_sol')(1);
        require('./restaking/3_deposit_token')(1);
        require('./restaking/11_operator_denormalize');

    } else if (process.env.JUST_RESTAKE) {
        require('./restaking/2_deposit_sol')(1);
        require('./restaking/3_deposit_token')(1);
        require('./restaking/9_operator_restaking');

    } else if (process.env.JUST_DELEGATE) {
        require('./restaking/2_deposit_sol')(1);
        require('./restaking/3_deposit_token')(1);
        require('./restaking/9_operator_restaking');
        require('./restaking/10_operator_restaking_delegation');

    } else if (process.env.JUST_WITHDRAW) {
        require('./restaking/4_withdraw_sol');

    } else if (process.env.JUST_TRANSFER) {
        require('./restaking/5_transfer_hook');

    } else if (process.env.JUST_WRAP) {
        require('./restaking/12_wrap');

    } else if (!process.env.JUST_INIT) {
        require('./restaking/2_deposit_sol')(1);
        require('./restaking/3_deposit_token')(1);
        require('./restaking/4_withdraw_sol');
        require('./restaking/5_transfer_hook');
        require('./restaking/6_reward');
        require('./restaking/12_wrap');

    }
}
