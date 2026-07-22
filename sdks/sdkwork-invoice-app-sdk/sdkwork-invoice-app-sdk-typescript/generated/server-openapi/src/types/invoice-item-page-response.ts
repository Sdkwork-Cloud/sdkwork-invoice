import type { InvoiceItem } from './invoice-item';
import type { PageInfo } from './page-info';

export interface InvoiceItemPageResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
