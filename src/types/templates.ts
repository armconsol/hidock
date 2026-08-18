// Template types

export interface Template {
  id: string;
  title: string;
  content: string;
  isFavorite: boolean;
  isDefault: boolean;
  createdAt: Date;
  updatedAt: Date;
  syncedAt?: Date;
}

export interface CreateTemplateRequest {
  title: string;
  content: string;
  isFavorite?: boolean;
  isDefault?: boolean;
}

export interface UpdateTemplateRequest {
  title?: string;
  content?: string;
  isFavorite?: boolean;
  isDefault?: boolean;
}

export interface TemplateFilter {
  searchQuery?: string;
  favoriteOnly?: boolean;
}

export type TemplateSortBy = 'createdAt' | 'updatedAt' | 'title';
export type SortOrder = 'asc' | 'desc';
