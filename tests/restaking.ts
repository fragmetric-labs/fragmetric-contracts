import * as chai from 'chai';
import chaiAsPromised from 'chai-as-promised';

chai.use(chaiAsPromised);
process.on('unhandledRejection', (err) => {
    // console.error(err);
    // process.exit(1);
})

/** define test suites here **/

require('./restaking/1_initialize');
// require('./restaking/2_deposit_sol');
// require('./restaking/3_deposit_token');
// require('./restaking/4_withdraw');
// require('./restaking/5_transfer_hook');
require('./restaking/6_reward');
