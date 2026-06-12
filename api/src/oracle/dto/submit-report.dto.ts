import { IsString, IsNotEmpty, IsNumber, IsPositive, Min, IsOptional } from 'class-validator';

export class SubmitReportDto {
  @IsString()
  @IsNotEmpty()
  projectId: string;

  @IsNumber()
  @IsPositive()
  periodStart: number;

  @IsNumber()
  @IsPositive()
  periodEnd: number;

  @IsNumber()
  @Min(0)
  carbonSequestered: number;

  @IsString()
  @IsNotEmpty()
  methodology: string;

  @IsString()
  @IsOptional()
  evidenceHash?: string;

  @IsNumber()
  nonce: number;
}
