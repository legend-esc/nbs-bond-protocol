import { IsString, IsNotEmpty, IsNumber, IsPositive, IsOptional, IsEnum } from 'class-validator';

export class ListBondDto {
  @IsNumber()
  @IsPositive()
  bondId: number;

  @IsNumber()
  @IsPositive()
  amount: number;

  @IsNumber()
  @IsPositive()
  pricePerToken: number;

  @IsString()
  @IsEnum(['USDC', 'XLM'])
  quoteAsset: 'USDC' | 'XLM';

  @IsNumber()
  @IsOptional()
  expiresAfterSeconds?: number = 604800;

  @IsNumber()
  nonce: number;
}
