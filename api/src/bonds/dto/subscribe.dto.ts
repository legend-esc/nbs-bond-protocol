import { IsNumber, IsPositive, IsString } from 'class-validator';
import { IsStellarAddress } from '../../common/decorators/is-stellar-address.decorator';

export class SubscribeDto {
  @IsNumber()
  @IsPositive()
  amount: number;

  @IsNumber()
  nonce: number;

  @IsString()
  @IsStellarAddress()
  investorAddress: string;
}
