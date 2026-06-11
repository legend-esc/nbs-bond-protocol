import { Module } from '@nestjs/common';
import { ProjectsController } from './projects.controller';
import { ProjectsService } from './projects.service';
import { IpfsService } from './ipfs.service';

@Module({
  controllers: [ProjectsController],
  providers: [ProjectsService, IpfsService],
})
export class ProjectsModule {}
