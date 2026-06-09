# Day 13 — Shared UI Library, Wallet Connect, Routing

Load context: `prompts/context/tech-stack.md`, `prompts/context/frontend-patterns.md`

## Goal

Build the shared component library, Stellar wallet connector, API service layer, and lazy-loaded routing shell for the Angular frontend.

## Files to Create

### Angular Config Files

#### `frontend/src/assets/.gitkeep`
Empty file to ensure the `assets/` directory is tracked by git.

#### `frontend/src/environments/environment.ts`
```typescript
export const environment = {
  production: false,
  apiUrl: '/api',
  stellarNetwork: 'testnet' as const,
  stellarHorizonUrl: 'https://horizon-testnet.stellar.org',
  networkPassphrase: 'Test SDF Network ; September 2015',
  ipfsGateway: 'https://gateway.pinata.cloud/ipfs/',
};
```

#### `frontend/src/environments/environment.prod.ts`
```typescript
export const environment = {
  production: true,
  apiUrl: '/api',
  stellarNetwork: 'mainnet' as const,
  stellarHorizonUrl: 'https://horizon.stellar.org',
  networkPassphrase: 'Public Global Stellar Network ; September 2015',
  ipfsGateway: 'https://gateway.pinata.cloud/ipfs/',
};
```

### Shared Components

#### `frontend/src/app/shared/components/bond-card/bond-card.component.ts`

Standalone component with:
- `@Input({ required: true }) bond: Bond`
- `@Output() subscribe = new EventEmitter<string>()` — emits bond ID
- Displays: bond ID, project name, face value, status badge, maturity date, coupon schedule summary
- Template uses `*ngIf` / `@if` for status-based styling (Active = green, Matured = blue, Defaulted = red)

#### `frontend/src/app/shared/components/project-card/project-card.component.ts`

Standalone component with:
- `@Input({ required: true }) project: Project`
- Displays: project name, methodology badge, country flag (emoji), status, area (ha), carbon estimate

#### `frontend/src/app/shared/components/status-badge/status-badge.component.ts`

Standalone component with:
- `@Input({ required: true }) status: string`
- `@Input() variant: 'bond' | 'project' | 'report' = 'bond'`
- Maps status to color: Active=#22c55e, Pending=#eab308, Matured=#3b82f6, Defaulted/Rejected=#ef4444, Verified=#22c55e
- Emits nothing — purely presentational

#### `frontend/src/app/shared/components/wallet-button/wallet-button.component.ts`

```typescript
@Component({
  selector: 'app-wallet-button',
  standalone: true,
  imports: [CommonModule],
  template: `
    <button *ngIf="!walletService.isConnected()" (click)="connect()">
      Connect Wallet
    </button>
    <button *ngIf="walletService.isConnected()" class="connected">
      {{ walletService.address()?.slice(0, 6) }}...{{ walletService.address()?.slice(-4) }}
    </button>
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WalletButtonComponent {
  readonly walletService = inject(WalletService);

  async connect(): Promise<void> {
    await this.walletService.connect();
  }
}
```

#### `frontend/src/app/shared/components/loading-spinner/loading-spinner.component.ts`

Simple spinner with `@Input() size: 'sm' | 'md' | 'lg' = 'md'`.

### Wallet Service

#### `frontend/src/app/auth/wallet.service.ts`

```typescript
@Injectable({ providedIn: 'root' })
export class WalletService {
  readonly address = signal<string | null>(null);
  readonly isConnected = signal(false);
  readonly isConnecting = signal(false);

  async connect(): Promise<void> {
    this.isConnecting.set(true);
    try {
      if (!await isConnected()) {
        throw new Error('Freighter not detected');
      }
      const pubKey = await getPublicKey();
      this.address.set(pubKey);
      this.isConnected.set(true);
    } finally {
      this.isConnecting.set(false);
    }
  }

  async signChallenge(challenge: string): Promise<string> {
    const tx = new TransactionBuilder(/* ... */)
      .addOperation(Operations.bumpSequence({ bumpTo: '0' }))
      .build();

    const signed = await signTransaction(tx.toXDR(), { networkPassphrase: Networks.TESTNET });
    return signed;
  }

  disconnect(): void {
    this.address.set(null);
    this.isConnected.set(false);
  }
}
```

### Auth Service & Login

#### `frontend/src/app/auth/auth.service.ts`

```typescript
@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly walletService = inject(WalletService);
  readonly token = signal<string | null>(localStorage.getItem('nbs_token'));
  readonly isAuthenticated = computed(() => this.token() !== null);

  async login(): Promise<void> {
    const address = this.walletService.address();
    if (!address) throw new Error('Wallet not connected');

    // 1. Get challenge
    const { challenge } = await firstValueFrom(
      this.http.post<{ challenge: string }>('/api/auth/challenge', { address })
    );

    // 2. Sign with Freighter
    const signedChallenge = await this.walletService.signChallenge(challenge);

    // 3. Verify and get JWT
    const { accessToken } = await firstValueFrom(
      this.http.post<{ accessToken: string }>('/api/auth/verify', {
        address, signedChallenge, originalChallenge: challenge,
      })
    );

    // 4. Store token
    localStorage.setItem('nbs_token', accessToken);
    this.token.set(accessToken);
  }

  logout(): void {
    localStorage.removeItem('nbs_token');
    this.token.set(null);
    this.walletService.disconnect();
  }
}
```

#### `frontend/src/app/auth/auth.component.ts`

Login page with:
- Wallet connect button (if not connected)
- "Sign In with Stellar" button (calls `authService.login()`)
- Shows wallet address after connection
- Redirects to `/dashboard` on success

#### `frontend/src/app/auth/auth.routes.ts`

```typescript
export default [
  { path: '', component: AuthComponent },
] as Routes;
```

### API Service

#### `frontend/src/app/shared/services/api.service.ts`

```typescript
@Injectable({ providedIn: 'root' })
export class ApiService {
  private readonly http = inject(HttpClient);
  private readonly authService = inject(AuthService);

  private headers(): HttpHeaders {
    const token = this.authService.token();
    return new HttpHeaders(token ? { Authorization: `Bearer ${token}` } : {});
  }

  // ── Bonds ──
  getBonds(page = 1, limit = 20): Observable<PaginatedResponse<Bond>> {
    return this.http.get<PaginatedResponse<Bond>>('/api/bonds', {
      params: { page, limit },
      headers: this.headers(),
    });
  }

  getBond(id: number): Observable<Bond> {
    return this.http.get<Bond>(`/api/bonds/${id}`, { headers: this.headers() });
  }

  subscribeToBond(id: number, amount: number, nonce: number): Observable<SubscriptionResponse> {
    return this.http.post<SubscriptionResponse>(
      `/api/bonds/${id}/subscribe`,
      { amount, nonce },
      { headers: this.headers() },
    );
  }

  // ── Projects ──
  getProjects(page = 1, limit = 20): Observable<PaginatedResponse<Project>> {
    return this.http.get<PaginatedResponse<Project>>('/api/projects', {
      params: { page, limit },
    });
  }

  getProject(id: number): Observable<Project> {
    return this.http.get<Project>(`/api/projects/${id}`);
  }

  registerProject(data: CreateProjectDto): Observable<Project> {
    return this.http.post<Project>('/api/projects', data, { headers: this.headers() });
  }

  // ── Marketplace ──
  getOrders(bondId?: number): Observable<Order[]> {
    const params: any = {};
    if (bondId) params.bondId = bondId;
    return this.http.get<Order[]>('/api/marketplace/orders', {
      params, headers: this.headers(),
    });
  }

  listBondTokens(data: ListBondDto): Observable<Order> {
    return this.http.post<Order>('/api/marketplace/list', data, { headers: this.headers() });
  }

  buyBondTokens(data: BuyBondDto): Observable<void> {
    return this.http.post<void>('/api/marketplace/buy', data, { headers: this.headers() });
  }
}
```

### Interfaces

#### `frontend/src/app/shared/interfaces/bond.interface.ts`

```typescript
export interface Bond {
  id: number;
  projectId: string;
  faceValue: number;
  couponSchedule: number[];
  creditType: 'Carbon' | 'Biodiversity' | 'Basket';
  maturityDate: number;
  totalSupply: number;
  totalSubscribed: number;
  status: 'Active' | 'Matured' | 'Defaulted';
  createdAt: string;
}

export interface Project {
  id: number;
  name: string;
  status: 'Pending' | 'Approved' | 'Rejected' | 'Inactive';
  methodology: string;
  country: string;
  metadataIpfsHash: string;
  ownerAddress: string;
  totalAreaHa: number;
  carbonSequestrationEstimate: number;
  createdAt: string;
}

export interface Order {
  id: number;
  seller: string;
  bondId: number;
  amount: number;
  pricePerToken: number;
  quoteAsset: 'USDC' | 'XLM';
  status: 'Open' | 'PartiallyFilled' | 'Filled' | 'Cancelled' | 'Expired';
  createdAt: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: { page: number; limit: number; total: number; totalPages: number };
}
```

### Routing Update

Update `frontend/src/app/app.routes.ts`:

```typescript
export const routes: Routes = [
  { path: '', redirectTo: '/dashboard', pathMatch: 'full' },
  {
    path: 'dashboard',
    loadComponent: () => import('./dashboard/dashboard.component').then(m => m.DashboardComponent),
  },
  {
    path: 'projects',
    loadChildren: () => import('./projects/projects.routes'),
  },
  {
    path: 'marketplace',
    loadChildren: () => import('./marketplace/marketplace.routes'),
  },
  {
    path: 'bonds',
    loadChildren: () => import('./bonds/bonds.routes'),
  },
  {
    path: 'auth',
    loadChildren: () => import('./auth/auth.routes'),
  },
  { path: '**', redirectTo: '/dashboard' },
];
```

Update `frontend/src/app/app.component.ts` template:

```html
<div class="app-shell">
  <header>
    <nav>
      <a routerLink="/dashboard">Dashboard</a>
      <a routerLink="/projects">Projects</a>
      <a routerLink="/bonds">Bonds</a>
      <a routerLink="/marketplace">Marketplace</a>
    </nav>
    <app-wallet-button />
  </header>
  <main><router-outlet /></main>
</div>
```

## Verification

```bash
cd frontend && npm run build
```

Expected: Build succeeds with zero errors. All lazy-loaded routes resolve.

```bash
cd frontend && npx ng test --watch=false --browsers=ChromeHeadless
```

Expected: 5+ passing tests covering wallet service, auth service, and shared components.

## Commit Message

```
feat(ui): shared components, wallet connect, API service, and lazy routing
```
