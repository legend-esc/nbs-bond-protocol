# Angular Patterns — Reference

## Component Template (Standalone)

```typescript
import { Component, input, output, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-bond-card',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './bond-card.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BondCardComponent {
  readonly bond = input.required<Bond>();
  readonly subscribe = output<string>();
}
```

## Service Template

```typescript
import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { toSignal } from '@angular/core/rxjs-interop';
import { map, tap } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class BondsService {
  private readonly http = inject(HttpClient);
  private readonly apiUrl = '/api/bonds';

  readonly bonds = signal<Bond[]>([]);

  loadAll(): void {
    this.http.get<Bond[]>(this.apiUrl).pipe(
      tap(bonds => this.bonds.set(bonds)),
    ).subscribe();
  }
}
```

## Wallet Connection Pattern

```typescript
import { Injectable, signal } from '@angular/core';
import { isConnected, getPublicKey, signTransaction } from '@stellar/freighter';

@Injectable({ providedIn: 'root' })
export class WalletService {
  readonly address = signal<string | null>(null);
  readonly isConnected = signal(false);

  async connect(): Promise<void> {
    if (await isConnected()) {
      const pubKey = await getPublicKey();
      this.address.set(pubKey);
      this.isConnected.set(true);
    }
  }
}
```

## State Management

Use signals + `computed` rather than NgRx for this phase:

```typescript
readonly activeBonds = computed(() =>
  this.bondsService.bonds().filter(b => b.status === 'active')
);
```

## Key Dependencies

```json
{
  "@angular/core": "^17.3.0",
  "@angular/common": "^17.3.0",
  "@angular/router": "^17.3.0",
  "@stellar/freighter": "^2.0.0",
  "@angular/forms": "^17.3.0"
}
```
