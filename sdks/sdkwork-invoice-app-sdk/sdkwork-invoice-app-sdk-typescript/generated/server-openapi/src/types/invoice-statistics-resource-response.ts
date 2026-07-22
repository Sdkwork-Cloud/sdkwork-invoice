import type { InvoiceStatistics } from './invoice-statistics';

export interface InvoiceStatisticsResourceResponse {
  code: 0;
  data: Record<string, unknown>;
  traceId: string;
}
