# Day 15 — Bond Issuance & Marketplace Trading UI

Load context: `prompts/context/tech-stack.md`, `prompts/context/frontend-patterns.md`

## Goal

Build the bond issuance/subscription flow and the DEX marketplace trading interface.

## Files to Create

### Bonds Module

#### `frontend/src/app/bonds/bonds.routes.ts`

```typescript
export default [
  { path: '', component: BondsListComponent },
  { path: ':id', component: BondDetailComponent },
  { path: 'issue', component: IssueBondComponent },
] as Routes;
```

#### `frontend/src/app/bonds/bonds-list.component.ts`

```typescript
@Component({
  selector: 'app-bonds-list',
  standalone: true,
  imports: [CommonModule, RouterModule, BondCardComponent, LoadingSpinnerComponent],
  templateUrl: './bonds-list.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BondsListComponent implements OnInit {
  readonly api = inject(ApiService);
  readonly bonds = signal<Bond[]>([]);
  readonly loading = signal(true);

  ngOnInit(): void {
    this.api.getBonds().subscribe({
      next: (res) => { this.bonds.set(res.data); this.loading.set(false); },
      error: () => this.loading.set(false),
    });
  }
}
```

Template: Grid of `<app-bond-card>` with a filter bar (All / Active / Matured).

#### `frontend/src/app/bonds/bond-detail.component.ts`

Full bond detail with coupon timeline and subscribe form:

```typescript
@Component({
  selector: 'app-bond-detail',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, StatusBadgeComponent, LoadingSpinnerComponent],
  templateUrl: './bond-detail.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BondDetailComponent implements OnInit {
  readonly api = inject(ApiService);
  readonly route = inject(ActivatedRoute);
  readonly authService = inject(AuthService);

  readonly bond = signal<Bond | null>(null);
  readonly loading = signal(true);
  readonly subscribing = signal(false);

  readonly subscribeForm = new FormGroup({
    amount: new FormControl(100, [Validators.required, Validators.min(1)]),
  });

  ngOnInit(): void {
    const id = Number(this.route.snapshot.paramMap.get('id'));
    this.api.getBond(id).subscribe({
      next: (b) => { this.bond.set(b); this.loading.set(false); },
      error: () => this.loading.set(false),
    });
  }

  onSubscribe(): void {
    if (this.subscribeForm.invalid || !this.bond()) return;
    this.subscribing.set(true);
    const { amount } = this.subscribeForm.value;
    this.api.subscribeToBond(this.bond()!.id, amount!, 0).subscribe({
      next: () => { this.subscribing.set(false); /* refresh */ },
      error: () => this.subscribing.set(false),
    });
  }
}
```

Template layout:
```
┌────────────────────────────────────────────┐
│  ← Back to Bonds                           │
├────────────────────────────────────────────┤
│  Bond #1                                    │
│  Status: [Active]  Credit Type: Carbon     │
├──────────┬─────────────────────────────────┤
│  Details │  Coupon Schedule                │
│  ─────── │  ─────────────────              │
│  Face    │  📅 Jan 15, 2026 (distributed)  │
│  Value:  │  📅 Apr 15, 2026 (upcoming)     │
│  $100K   │  📅 Jul 15, 2026 (upcoming)     │
│  Supply: │  📅 Oct 15, 2026 (upcoming)     │
│  1,000   │                                  │
│  Subscri │  ────────────────                │
│  bed: 750│  Total Accrued Credits: 12,500  │
│  Maturit │                                  │
│  y:      │                                  │
│  2027-01 │                                  │
├──────────┴─────────────────────────────────┤
│  Subscribe                                 │
│  ┌──────────────────────┐ ┌──────────────┐ │
│  │ Amount: [  100     ] │ │ [Subscribe]  │ │
│  └──────────────────────┘ └──────────────┘ │
│  (requires KYC)                             │
├─────────────────────────────────────────────┤
│  Holders (5)                                │
│  GABCD...1234 — 300 tokens                  │
│  GXYZT...5678 — 200 tokens                  │
│  ...                                        │
└─────────────────────────────────────────────┘
```

#### `frontend/src/app/bonds/issue-bond.component.ts`

Form for bond issuance (admin/issuer role):

```typescript
@Component({
  selector: 'app-issue-bond',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule],
  templateUrl: './issue-bond.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class IssueBondComponent {
  readonly api = inject(ApiService);
  readonly submitting = signal(false);

  readonly form = new FormGroup({
    projectId: new FormControl('', Validators.required),
    faceValue: new FormControl(0, [Validators.required, Validators.min(1)]),
    creditType: new FormControl('Carbon', Validators.required),
    maturityDate: new FormControl('', Validators.required),
    totalSupply: new FormControl(1000, [Validators.required, Validators.min(1)]),
  });
}
```

Fields: project ID (text), face value (number), credit type (select: Carbon/Biodiversity/Basket), maturity date (date picker), total supply (number).

### Marketplace Module

#### `frontend/src/app/marketplace/marketplace.routes.ts`

```typescript
export default [
  { path: '', component: MarketplaceComponent },
] as Routes;
```

#### `frontend/src/app/marketplace/marketplace.component.ts`

```typescript
@Component({
  selector: 'app-marketplace',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, LoadingSpinnerComponent],
  templateUrl: './marketplace.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MarketplaceComponent implements OnInit {
  readonly api = inject(ApiService);
  readonly authService = inject(AuthService);

  readonly orders = signal<Order[]>([]);
  readonly bonds = signal<Bond[]>([]);
  readonly loading = signal(true);
  readonly selectedBondId = signal<number | null>(null);

  // Price data (computed)
  readonly bestPrices = computed(() => {
    const orders = this.orders();
    const grouped = new Map<number, Order[]>();
    orders.filter(o => o.status === 'Open').forEach(o => {
      const list = grouped.get(o.bondId) || [];
      list.push(o);
      grouped.set(o.bondId, list);
    });
    const result: Record<number, { best: number; average: number }> = {};
    grouped.forEach((list, bondId) => {
      const prices = list.map(o => o.pricePerToken);
      result[bondId] = {
        best: Math.min(...prices),
        average: prices.reduce((a, b) => a + b, 0) / prices.length,
      };
    });
    return result;
  });

  readonly filteredOrders = computed(() => {
    const selected = this.selectedBondId();
    return selected ? this.orders().filter(o => o.bondId === selected) : this.orders();
  });

  ngOnInit(): void {
    // Load bonds and orders in parallel
    forkFetch();
  }

  // Buy modal state
  readonly showBuyModal = signal(false);
  readonly selectedOrder = signal<Order | null>(null);

  openBuyModal(order: Order): void {
    this.selectedOrder.set(order);
    this.showBuyModal.set(true);
  }
}
```

Template layout:
```
┌──────────────────────────────────────────────┐
│  Marketplace  [Filter by Bond ▼]             │
├──────────────────────────────────────────────┤
│  Price Overview                              │
│  ┌──────────┬──────────┬──────────────────┐  │
│  │ Bond #1  │ Best: 5  │ Avg: 5.2 USDC   │  │
│  │ Bond #3  │ Best: 8  │ Avg: 8.1 USDC   │  │
│  └──────────┴──────────┴──────────────────┘  │
├──────────────────────────────────────────────┤
│  Open Orders (12)    [+ Create Listing]      │
│  ┌──────┬──────┬────┬──────┬──────┬──────┐  │
│  │ Bond │Seller│Amt │Price │Total │Action│  │
│  ├──────┼──────┼────┼──────┼──────┼──────┤  │
│  │ #1   │GABC..│100 │5 USDC│500   │[Buy] │  │
│  │ #1   │GXYZ..│200 │6 USDC│1200  │[Buy] │  │
│  │ #3   │G123..│50  │8 USDC│400   │[Buy] │  │
│  └──────┴──────┴────┴──────┴──────┴──────┘  │
├──────────────────────────────────────────────┤
│  My Orders (if connected)                    │
│  ┌──────┬──────┬──────┬────────┬─────────┐   │
│  │ Bond │Amt   │Price │Status  │ Action  │   │
│  ├──────┼──────┼──────┼────────┼─────────┤   │
│  │ #1   │50    │5     │Open    │[Cancel] │   │
│  └──────┴──────┴──────┴────────┴─────────┘   │
└──────────────────────────────────────────────┘
```

#### Create Listing Modal

Form fields:
- Bond ID (select from owned bonds)
- Amount (number, min 1)
- Price per token (number, in USDC)
- Expires (select: 1 day / 7 days / 30 days)

#### Buy Modal

Shows order details and has confirm button:
- Order ID, Seller, Amount, Price, Total
- "Confirm Purchase" button

## Verification

```bash
cd frontend && npm run build
cd frontend && npx ng test --watch=false --browsers=ChromeHeadless --include='src/app/bonds/**/*.spec.ts'
cd frontend && npx ng test --watch=false --browsers=ChromeHeadless --include='src/app/marketplace/**/*.spec.ts'
```

Expected: Build succeeds. 10+ tests — bond detail renders, subscribe form validates, marketplace loads orders, buy modal, create listing form.

## Commit Message

```
feat(ui): bond issuance/subscription flow and DEX marketplace trading interface
```
