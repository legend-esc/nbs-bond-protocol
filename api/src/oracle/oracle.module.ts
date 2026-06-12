import { Module } from '@nestjs/common';
import { ScheduleModule } from '@nestjs/schedule';
import { OracleController } from './oracle.controller';
import { OracleService } from './oracle.service';
import { OracleScheduler } from './oracle.scheduler';
import { VerraProvider } from './providers/verra.provider';
import { SatelliteProvider } from './providers/satellite.provider';

@Module({
  imports: [ScheduleModule.forRoot()],
  controllers: [OracleController],
  providers: [
    OracleService,
    OracleScheduler,
    VerraProvider,
    SatelliteProvider,
  ],
  exports: [OracleService],
})
export class OracleModule {}
