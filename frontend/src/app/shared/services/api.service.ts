import { Injectable, inject } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';
import { AuthService } from '../../auth/auth.service';
import {
  Bond, Project, Order, PaginatedResponse,
  SubscriptionResponse, CreateProjectDto, ListBondDto, BuyBondDto,
} from '../interfaces/bond.interface';

@Injectable({ providedIn: 'root' })
export class ApiService {
  private readonly http = inject(HttpClient);
  private readonly authService = inject(AuthService);

  private headers(): HttpHeaders {
    const token = this.authService.token();
    return new HttpHeaders(token ? { Authorization: `Bearer ${token}` } : {});
  }

  getBonds(page = 1, limit = 20): Observable<PaginatedResponse<Bond>> {
    return this.http.get<PaginatedResponse<Bond>>('/api/bonds', {
      params: { page, limit },
      headers: this.headers(),
    });
  }

  getBond(id: number): Observable<Bond> {
    return this.http.get<Bond>(`/api/bonds/${id}`, { headers: this.headers() });
  }

  issueBond(data: any): Observable<Bond> {
    return this.http.post<Bond>('/api/bonds', data, { headers: this.headers() });
  }

  subscribeToBond(id: number, amount: number, nonce: number): Observable<SubscriptionResponse> {
    return this.http.post<SubscriptionResponse>(
      `/api/bonds/${id}/subscribe`,
      { amount, nonce },
      { headers: this.headers() },
    );
  }

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
