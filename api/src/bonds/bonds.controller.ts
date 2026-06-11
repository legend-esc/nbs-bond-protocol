import {
  Controller, Get, Post, Body, Param, Query, Req,
  HttpCode, HttpStatus, UseGuards, ParseIntPipe,
} from '@nestjs/common';
import { BondsService } from './bonds.service';
import { CreateBondDto } from './dto/create-bond.dto';
import { SubscribeDto } from './dto/subscribe.dto';
import { DistributeCouponDto } from './dto/distribute-coupon.dto';
import { PaginationDto } from '../common/dto/pagination.dto';
import {
  BondResponse,
  SubscriptionResponse,
  HolderListResponse,
  CouponDistributionResponse,
} from './interfaces/bond.interface';

@Controller('bonds')
export class BondsController {
  constructor(private readonly bondsService: BondsService) {}

  @Post()
  @HttpCode(HttpStatus.CREATED)
  async create(@Body() dto: CreateBondDto): Promise<BondResponse> {
    return this.bondsService.create(dto);
  }

  @Get()
  async findAll(@Query() query: PaginationDto) {
    return this.bondsService.findAll(query.page, query.limit);
  }

  @Get(':id')
  async findOne(@Param('id', ParseIntPipe) id: number): Promise<BondResponse> {
    return this.bondsService.findOne(id);
  }

  @Post(':id/subscribe')
  @HttpCode(HttpStatus.OK)
  async subscribe(
    @Param('id', ParseIntPipe) id: number,
    @Body() dto: SubscribeDto,
  ): Promise<SubscriptionResponse> {
    return this.bondsService.subscribe(id, dto);
  }

  @Get(':id/holders')
  async getHolders(
    @Param('id', ParseIntPipe) id: number,
  ): Promise<HolderListResponse> {
    return this.bondsService.getHolders(id);
  }

  @Post(':id/coupon')
  @HttpCode(HttpStatus.OK)
  async distributeCoupon(
    @Param('id', ParseIntPipe) id: number,
    @Body() dto: DistributeCouponDto,
  ): Promise<CouponDistributionResponse> {
    return this.bondsService.distributeCoupon(id, dto);
  }

  @Post(':id/mature')
  @HttpCode(HttpStatus.OK)
  async mature(
    @Param('id', ParseIntPipe) id: number,
  ): Promise<BondResponse> {
    return this.bondsService.mature(id);
  }
}
