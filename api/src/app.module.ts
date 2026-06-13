import { Module } from '@nestjs/common';
import { APP_FILTER, APP_INTERCEPTOR } from '@nestjs/core';
import { BondsModule } from './bonds/bonds.module';
import { ProjectsModule } from './projects/projects.module';
import { OracleModule } from './oracle/oracle.module';
import { MarketplaceModule } from './marketplace/marketplace.module';
import { AuthModule } from './auth/auth.module';
import { StellarModule } from './stellar/stellar.module';
import { Rfc7807ExceptionFilter } from './common/filters/rfc7807-exception.filter';
import { RequestLoggingInterceptor } from './common/interceptors/request-logging.interceptor';

@Module({
  imports: [
    BondsModule,
    ProjectsModule,
    OracleModule,
    MarketplaceModule,
    AuthModule,
    StellarModule,
  ],
  providers: [
    { provide: APP_FILTER, useClass: Rfc7807ExceptionFilter },
    { provide: APP_INTERCEPTOR, useClass: RequestLoggingInterceptor },
  ],
})
export class AppModule {}
