import { Module } from '@nestjs/common';
import { BondsModule } from './bonds/bonds.module';
import { ProjectsModule } from './projects/projects.module';
import { OracleModule } from './oracle/oracle.module';
import { MarketplaceModule } from './marketplace/marketplace.module';
import { AuthModule } from './auth/auth.module';
import { StellarModule } from './stellar/stellar.module';

@Module({
  imports: [
    BondsModule,
    ProjectsModule,
    OracleModule,
    MarketplaceModule,
    AuthModule,
    StellarModule,
  ],
})
export class AppModule {}
