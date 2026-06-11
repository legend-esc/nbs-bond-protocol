import { Global, Module } from '@nestjs/common';
import { StellarService } from './stellar.service';
import { ContractService } from './contract.service';

@Global()
@Module({
  providers: [StellarService, ContractService],
  exports: [StellarService, ContractService],
})
export class StellarModule {}
