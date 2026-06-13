import {
  Injectable,
  NestInterceptor,
  ExecutionContext,
  CallHandler,
  Logger,
} from '@nestjs/common';
import { Observable } from 'rxjs';
import { tap } from 'rxjs/operators';
import * as crypto from 'crypto';

@Injectable()
export class RequestLoggingInterceptor implements NestInterceptor {
  private readonly logger = new Logger('HTTP');

  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    const request = context.switchToHttp().getRequest();
    const requestId = crypto.randomUUID();
    request.requestId = requestId;

    const start = Date.now();
    this.logger.log(`[${requestId}] ${request.method} ${request.url}`);

    return next.handle().pipe(
      tap({
        next: () => this.logger.log(`[${requestId}] ${Date.now() - start}ms`),
        error: (err) => this.logger.error(`[${requestId}] ${err.message}`, err.stack),
      }),
    );
  }
}
