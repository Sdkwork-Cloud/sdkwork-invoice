import type { Invoice } from './invoice';

export interface InvoiceResourceResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
