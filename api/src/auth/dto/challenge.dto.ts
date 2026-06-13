import { IsString, IsNotEmpty } from 'class-validator';
import { IsStellarAddress } from '../../common/decorators/is-stellar-address.decorator';

export class ChallengeDto {
  @IsString()
  @IsNotEmpty()
  @IsStellarAddress()
  address: string;
}
