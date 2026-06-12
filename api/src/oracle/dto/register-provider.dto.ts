import { IsString, IsNotEmpty } from 'class-validator';
import { IsStellarAddress } from '../../common/decorators/is-stellar-address.decorator';

export class RegisterProviderDto {
  @IsString()
  @IsStellarAddress()
  providerAddress: string;

  @IsString()
  @IsNotEmpty()
  methodology: string;
}
