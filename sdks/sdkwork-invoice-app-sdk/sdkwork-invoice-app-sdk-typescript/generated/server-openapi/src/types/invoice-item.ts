export interface InvoiceItem {
  id: string;
  invoiceId: string;
  orderItemId?: string | null;
  title: string;
  amount: string;
  taxAmount: string;
  createdAt: string;
}
