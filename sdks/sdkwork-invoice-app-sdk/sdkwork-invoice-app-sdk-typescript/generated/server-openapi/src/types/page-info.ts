export interface PageInfo {
  mode: 'offset' | 'cursor';
  page?: number | null;
  pageSize?: number | null;
  totalItems?: string | null;
  totalPages?: number | null;
  nextCursor?: string | null;
  hasMore?: boolean | null;
}
