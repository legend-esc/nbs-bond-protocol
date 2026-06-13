import {
  Controller, Get, Post, Body, Req,
  HttpCode, HttpStatus, UseGuards,
} from '@nestjs/common';
import { AuthService } from './auth.service';
import { ChallengeDto } from './dto/challenge.dto';
import { VerifySignatureDto } from './dto/verify-signature.dto';
import { RefreshTokenDto } from './dto/refresh-token.dto';
import { JwtAuthGuard } from '../common/guards/jwt-auth.guard';
import { AuthenticatedRequest } from '../common/interfaces/authenticated-request.interface';
import {
  ChallengeResponse,
  AuthTokenResponse,
  UserProfileResponse,
} from './interfaces/auth.interface';

@Controller('auth')
export class AuthController {
  constructor(private readonly authService: AuthService) {}

  @Post('challenge')
  @HttpCode(HttpStatus.OK)
  async challenge(@Body() dto: ChallengeDto): Promise<ChallengeResponse> {
    return this.authService.generateChallenge(dto.address);
  }

  @Post('verify')
  @HttpCode(HttpStatus.OK)
  async verify(@Body() dto: VerifySignatureDto): Promise<AuthTokenResponse> {
    return this.authService.verifySignature(dto);
  }

  @Post('refresh')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  async refresh(@Body() dto: RefreshTokenDto): Promise<AuthTokenResponse> {
    return this.authService.refreshToken(dto.accessToken);
  }

  @Get('profile')
  @UseGuards(JwtAuthGuard)
  async profile(@Req() req: AuthenticatedRequest): Promise<UserProfileResponse> {
    return this.authService.getProfile(req.user.walletAddress);
  }
}
