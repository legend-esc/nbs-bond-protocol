import { IsString, IsNotEmpty, IsNumber } from 'class-validator';

export class ChallengeDto {
  @IsString()
  @IsNotEmpty()
  counterEvidenceHash: string;

  @IsString()
  @IsNotEmpty()
  reason: string;

  @IsNumber()
  nonce: number;
}
