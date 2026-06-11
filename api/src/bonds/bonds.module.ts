import { Module } from '@nestjs/common';
import { BondsController } from './bonds.controller';
import { BondsService } from './bonds.service';

@Module({
  controllers: [BondsController],
  providers: [BondsService],
})
export class BondsModule {}
