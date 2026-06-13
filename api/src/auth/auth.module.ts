import { Module } from '@nestjs/common';
import { JwtModule } from '@nestjs/jwt';
import { PassportModule } from '@nestjs/passport';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { JwtStrategy } from './jwt.strategy';
import { KycService } from './kyc.service';

@Module({
  imports: [
    PassportModule.register({ defaultStrategy: 'jwt' }),
    JwtModule.register({
      secret: process.env.JWT_SECRET || 'dev-secret-change-in-production',
      signOptions: { expiresIn: process.env.JWT_EXPIRY || '7d' },
    }),
  ],
  controllers: [AuthController],
  providers: [AuthService, JwtStrategy, KycService],
  exports: [AuthService, KycService],
})
export class AuthModule {}
