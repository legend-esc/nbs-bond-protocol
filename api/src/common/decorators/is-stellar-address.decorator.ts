import { registerDecorator, ValidationOptions } from 'class-validator';

export function IsStellarAddress(validationOptions?: ValidationOptions) {
  return function (object: object, propertyName: string) {
    registerDecorator({
      name: 'isStellarAddress',
      target: object.constructor,
      propertyName,
      options: validationOptions,
      validator: {
        validate(value: string) {
          return typeof value === 'string' && /^G[A-Z0-9]{55}$/.test(value);
        },
        defaultMessage: () =>
          'Invalid Stellar address (must start with G and be 56 chars)',
      },
    });
  };
}
