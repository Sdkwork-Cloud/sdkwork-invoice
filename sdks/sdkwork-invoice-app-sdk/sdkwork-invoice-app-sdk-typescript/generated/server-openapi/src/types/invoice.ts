import type { InvoiceItem } from './invoice-item';

export interface Invoice {
  id: string;
  orderId: string;
  paymentId: string;
  titleId: string;
  status: string;
  invoiceNo?: string | null;
  invoiceCode?: string | null;
  documentUrl?: string;
  createdAt: string;
  issuedAt?: string | null;
  updatedAt: string;
  totalAmount: string;
  items: InvoiceItem[];
}
