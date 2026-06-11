import { IsNumber, IsPositive, IsObject, IsString, ValidateNested } from 'class-validator';
import { Type } from 'class-transformer';

export class OracleReportDto {
  @IsString()
  projectId: string;

  @IsNumber()
  periodStart: number;

  @IsNumber()
  periodEnd: number;

  @IsNumber()
  carbonSequestered: number;

  @IsString()
  methodology: string;

  @IsString()
  providerSignature: string;

  @IsString()
  ipfsEvidenceHash: string;
}

export class DistributeCouponDto {
  @IsNumber()
  @IsPositive()
  periodIndex: number;

  @IsObject()
  @ValidateNested()
  @Type(() => OracleReportDto)
  report: OracleReportDto;
}
