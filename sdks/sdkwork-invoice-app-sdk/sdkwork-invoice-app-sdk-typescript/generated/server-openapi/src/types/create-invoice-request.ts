export interface CreateInvoiceRequest {
  title: string;
  taxNo?: string | null;
  titleType?: string | null;
  totalAmount?: string | number;
  type?: string | null;
}
