import { IsString, IsNotEmpty } from 'class-validator';
import { IsStellarAddress } from '../../common/decorators/is-stellar-address.decorator';

export class VerifySignatureDto {
  @IsString()
  @IsNotEmpty()
  @IsStellarAddress()
  address: string;

  @IsString()
  @IsNotEmpty()
  signedChallenge: string;

  @IsString()
  @IsNotEmpty()
  originalChallenge: string;
}
