import type { Invoice } from './invoice';
import type { PageInfo } from './page-info';

export interface InvoicePageResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
