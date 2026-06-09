# Day 14 — Dashboard & Project Registry Pages

Load context: `prompts/context/tech-stack.md`, `prompts/context/frontend-patterns.md`

## Goal

Build the investor dashboard (portfolio view) and project registry pages with IPFS document viewer.

## Files to Create

### Dashboard Module

#### `frontend/src/app/dashboard/dashboard.component.ts`

```typescript
@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, RouterModule, BondCardComponent, LoadingSpinnerComponent],
  templateUrl: './dashboard.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly walletService = inject(WalletService);

  readonly bonds = signal<Bond[]>([]);
  readonly loading = signal(true);
  readonly walletAddress = this.walletService.address;

  // Computed values for portfolio summary
  readonly totalInvested = computed(() =>
    this.bonds().reduce((sum, b) => sum + b.totalSubscribed, 0)
  );
  readonly activeBonds = computed(() =>
    this.bonds().filter(b => b.status === 'Active')
  );
  readonly maturedBonds = computed(() =>
    this.bonds().filter(b => b.status === 'Matured')
  );

  ngOnInit(): void {
    this.api.getBonds().subscribe({
      next: (res) => {
        this.bonds.set(res.data);
        this.loading.set(false);
      },
      error: () => this.loading.set(false),
    });
  }
}
```

#### `frontend/src/app/dashboard/dashboard.component.html`

Layout:
```
┌──────────────────────────────────────────┐
│  Dashboard                               │
│  Wallet: GABCD...1234                    │
├──────────────────────────────────────────┤
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌─────────┐│
│  │Total │ │Active│ │Matured│ │Accrued  ││
│  │Invest│ │Bonds │ │Bonds  │ │Credits  ││
│  │$X    │ │  Y   │ │   Z   │ │    W    ││
│  └──────┘ └──────┘ └──────┘ └─────────┘│
├──────────────────────────────────────────┤
│  Active Bonds                            │
│  ┌──────────────────────────────────────┐│
│  │ <app-bond-card *ngFor="..." />      ││
│  └──────────────────────────────────────┘│
├──────────────────────────────────────────┤
│  Recent Activity                         │
│  - Subscribed to Bond #1 (2h ago)       │
│  - Coupon distributed for Bond #3 (1d)  │
└──────────────────────────────────────────┘
```

- Use CSS grid or flexbox for stat cards
- Show "Connect Wallet to View Portfolio" banner when wallet is not connected
- Recent activity is static placeholder data for now

### Projects Module

#### `frontend/src/app/projects/projects.routes.ts`

```typescript
export default [
  { path: '', component: ProjectsListComponent },
  { path: ':id', component: ProjectDetailComponent },
] as Routes;
```

#### `frontend/src/app/projects/projects-list.component.ts`

```typescript
@Component({
  selector: 'app-projects-list',
  standalone: true,
  imports: [CommonModule, ProjectCardComponent, LoadingSpinnerComponent, RouterModule],
  templateUrl: './projects-list.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProjectsListComponent implements OnInit {
  readonly api = inject(ApiService);
  readonly projects = signal<Project[]>([]);
  readonly loading = signal(true);
  readonly filter = signal<'all' | 'approved' | 'pending'>('all');

  readonly filteredProjects = computed(() => {
    const f = this.filter();
    if (f === 'all') return this.projects();
    return this.projects().filter(p => p.status.toLowerCase() === f);
  });

  ngOnInit(): void {
    this.api.getProjects().subscribe({
      next: (res) => { this.projects.set(res.data); this.loading.set(false); },
      error: () => this.loading.set(false),
    });
  }
}
```

Template:
```
┌─────────────────────────────────────────────┐
│  Projects  [All] [Approved] [Pending]       │
│  [+ Register New Project] (if wallet conn)  │
├─────────────────────────────────────────────┤
│  ┌────────────────────┐  ┌────────────────┐ │
│  │ <app-project-card> │  │ <app-project>  │ │
│  └────────────────────┘  └────────────────┘ │
│  ┌────────────────────┐  ┌────────────────┐ │
│  │ ...                │  │                │ │
└─────────────────────────────────────────────┘
```

Use CSS grid: 2 columns on desktop, 1 on mobile.

#### `frontend/src/app/projects/project-detail.component.ts`

Shows full project information:

```typescript
@Component({
  selector: 'app-project-detail',
  standalone: true,
  imports: [CommonModule, RouterModule, LoadingSpinnerComponent, StatusBadgeComponent],
  templateUrl: './project-detail.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProjectDetailComponent implements OnInit {
  private readonly api = inject(ApiService);
  readonly route = inject(ActivatedRoute);

  readonly project = signal<Project | null>(null);
  readonly loading = signal(true);

  // Computed detail values
  readonly documents = computed(() => {
    // Placeholder: would fetch from IPFS via project.metadataIpfsHash
    return [];
  });

  ngOnInit(): void {
    const id = Number(this.route.snapshot.paramMap.get('id'));
    this.api.getProject(id).subscribe({
      next: (p) => { this.project.set(p); this.loading.set(false); },
      error: () => this.loading.set(false),
    });
  }
}
```

Template layout:
```
┌──────────────────────────────────────────┐
│  ← Back to Projects                      │
├──────────────────────────────────────────┤
│  Project Name (from IPFS metadata)       │
│  Status: [Approved]    ID: #1            │
├──────────────────────────────────────────┤
│  Details           │  Documents          │
│  ─────────         │  ─────────          │
│  Country: Brazil   │  📄 Prospectus      │
│  Methodology: VCS  │  📄 Audit Report    │
│  Area: 1,200 ha    │  📄 Satellite Data  │
│  Est. Sequestration│                     │
│  : 50,000 tCO2e/yr │                     │
├──────────────────────────────────────────┤
│  Oracle History                          │
│  ┌────┬────────┬──────────┬──────────┐   │
│  │ Per│ Period │ Credits  │ Status   │   │
│  ├────┼────────┼──────────┼──────────┤   │
│  │ 1  │ Q1 '25 │ 12,500   │ Verified │   │
│  │ 2  │ Q2 '25 │ 13,200   │ Pending  │   │
│  └────┴────────┴──────────┴──────────┘   │
└──────────────────────────────────────────┘
```

### Register Project Modal

#### `frontend/src/app/projects/register-project.component.ts`

```typescript
@Component({
  selector: 'app-register-project',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule],
  template: `...`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class RegisterProjectComponent {
  private readonly api = inject(ApiService);
  private readonly walletService = inject(WalletService);
  private readonly authService = inject(AuthService);

  readonly form = new FormGroup({
    name: new FormControl('', Validators.required),
    methodology: new FormControl('', Validators.required),
    country: new FormControl('', Validators.required),
    totalAreaHa: new FormControl(0, [Validators.required, Validators.min(1)]),
    carbonSequestrationEstimate: new FormControl(0, [Validators.required, Validators.min(0)]),
    description: new FormControl(''),
    blueCarbon: new FormControl(false),
    biodiversityCorridor: new FormControl(false),
  });

  readonly submitted = signal(false);

  async onSubmit(): Promise<void> {
    if (this.form.invalid) return;
    this.submitted.set(true);

    const dto = { ...this.form.value, nonce: 0 };
    this.api.registerProject(dto).subscribe({
      next: () => { /* success — close modal, refresh list */ },
      error: () => this.submitted.set(false),
    });
  }
}
```

Form fields use `<input formControlName="...">` with validation error messages below each field.

## Verification

```bash
cd frontend && npm run build
cd frontend && npx ng test --watch=false --browsers=ChromeHeadless --include='src/app/dashboard/**/*.spec.ts'
cd frontend && npx ng test --watch=false --browsers=ChromeHeadless --include='src/app/projects/**/*.spec.ts'
```

Expected: Build succeeds. 8+ tests — dashboard component renders stats, project list filters, project detail loads, form validation.

## Commit Message

```
feat(ui): dashboard portfolio view and project registry pages with IPFS detail view
```
