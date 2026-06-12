import { Module } from '@nestjs/common';
import { MarketplaceController } from './marketplace.controller';
import { DexService } from './dex.service';
import { LiquidityService } from './liquidity.service';

@Module({
  controllers: [MarketplaceController],
  providers: [DexService, LiquidityService],
  exports: [DexService, LiquidityService],
})
export class MarketplaceModule {}
