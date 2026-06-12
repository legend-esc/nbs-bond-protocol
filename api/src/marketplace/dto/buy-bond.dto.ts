import { IsNumber, IsPositive } from 'class-validator';

export class BuyBondDto {
  @IsNumber()
  @IsPositive()
  orderId: number;

  @IsNumber()
  @IsPositive()
  amount: number;

  @IsNumber()
  @IsPositive()
  maxPrice: number;

  @IsNumber()
  nonce: number;
}
