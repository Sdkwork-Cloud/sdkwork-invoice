export interface InvoiceCommand {
  accepted: boolean;
  resourceId?: string | null;
  status?: string | null;
}
