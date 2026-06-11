import {
  IsString,
  IsNotEmpty,
  IsNumber,
  IsPositive,
  IsArray,
  IsEnum,
} from 'class-validator';
import { CreditTypeEnum } from '../interfaces/bond.interface';

export class CreateBondDto {
  @IsString()
  @IsNotEmpty()
  projectId: string;

  @IsNumber()
  @IsPositive()
  faceValue: number;

  @IsArray()
  @IsNumber({}, { each: true })
  couponSchedule: number[];

  @IsEnum(CreditTypeEnum)
  creditType: CreditTypeEnum;

  @IsNumber()
  @IsPositive()
  maturityDate: number;

  @IsNumber()
  @IsPositive()
  totalSupply: number;
}
