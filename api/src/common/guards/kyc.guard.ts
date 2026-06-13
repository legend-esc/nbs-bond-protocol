import { Injectable, CanActivate, ExecutionContext, UnauthorizedException, ForbiddenException } from '@nestjs/common';
import { KycService } from '../../auth/kyc.service';
import { KycStatus, AuthenticatedUser } from '../interfaces/authenticated-request.interface';

@Injectable()
export class KycGuard implements CanActivate {
  constructor(private readonly kycService: KycService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest();
    const user = request.user as AuthenticatedUser;

    if (!user) {
      throw new UnauthorizedException('Authentication required');
    }

    const eligible = await this.kycService.isEligible(user.walletAddress, KycStatus.VERIFIED);
    if (!eligible) {
      throw new ForbiddenException('KYC verification required');
    }

    return true;
  }
}
