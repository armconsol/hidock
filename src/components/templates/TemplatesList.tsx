import { useEffect } from 'react';
import {
  Space,
  Input,
  Button,
  Select,
  Empty,
  Spin,
  Card,
  Typography,
  Tag,
  Popconfirm,
  Message,
} from '@arco-design/web-react';
import {
  IconSearch,
  IconPlus,
  IconSort,
  IconStar,
  IconStarFill,
  IconCheck,
  IconEdit,
  IconDelete,
} from '@arco-design/web-react/icon';
import { useTemplatesStore } from '../../store/templatesStore';
import type { TemplateSortBy, SortOrder } from '../../types/templates';
import './TemplatesList.css';

const { Option } = Select;
const { Title, Text } = Typography;

export function TemplatesList() {
  const {
    selectedTemplateId,
    filter,
    sortBy,
    sortOrder,
    isLoading,
    error,
    loadTemplates,
    selectTemplate,
    setFilter,
    setSorting,
    toggleFavorite,
    setAsDefault,
    deleteTemplate,
    getFilteredTemplates,
  } = useTemplatesStore();

  useEffect(() => {
    loadTemplates();
  }, []);

  const filteredTemplates = getFilteredTemplates();

  const handleSearch = (value: string) => {
    setFilter({ ...filter, searchQuery: value });
  };

  const handleSortChange = (value: string) => {
    const [newSortBy, newSortOrder] = value.split('-') as [TemplateSortBy, SortOrder];
    setSorting(newSortBy, newSortOrder);
  };

  const handleToggleFavorite = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await toggleFavorite(id);
      Message.success('Favorite status updated');
    } catch (err) {
      Message.error('Failed to update favorite status');
    }
  };

  const handleSetDefault = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await setAsDefault(id);
      Message.success('Default template updated');
    } catch (err) {
      Message.error('Failed to set default template');
    }
  };

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await deleteTemplate(id);
      Message.success('Template deleted');
    } catch (err) {
      Message.error('Failed to delete template');
    }
  };

  const handleFilterFavorites = () => {
    setFilter({ ...filter, favoriteOnly: !filter.favoriteOnly });
  };

  const sortOptions = [
    { label: 'Last Modified (Newest)', value: 'updatedAt-desc' },
    { label: 'Last Modified (Oldest)', value: 'updatedAt-asc' },
    { label: 'Date Created (Newest)', value: 'createdAt-desc' },
    { label: 'Date Created (Oldest)', value: 'createdAt-asc' },
    { label: 'Title (A-Z)', value: 'title-asc' },
    { label: 'Title (Z-A)', value: 'title-desc' },
  ];

  if (error) {
    return (
      <div className="templates-list-error">
        <Text type="error">{error}</Text>
      </div>
    );
  }

  return (
    <div className="templates-list">
      <div className="templates-list-header">
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div className="templates-list-actions">
            <Button
              type="primary"
              icon={<IconPlus />}
              onClick={() => selectTemplate('new')}
              style={{ width: '100%' }}
            >
              New Template
            </Button>
          </div>
          <Input
            allowClear
            placeholder="Search templates..."
            prefix={<IconSearch />}
            value={filter.searchQuery || ''}
            onChange={handleSearch}
          />
          <Space size={8} style={{ width: '100%' }}>
            <Select
              placeholder="Sort by"
              value={`${sortBy}-${sortOrder}`}
              onChange={handleSortChange}
              style={{ flex: 1 }}
              prefix={<IconSort />}
            >
              {sortOptions.map((option) => (
                <Option key={option.value} value={option.value}>
                  {option.label}
                </Option>
              ))}
            </Select>
            <Button
              icon={filter.favoriteOnly ? <IconStarFill /> : <IconStar />}
              type={filter.favoriteOnly ? 'primary' : 'default'}
              onClick={handleFilterFavorites}
            />
          </Space>
        </Space>
      </div>

      <div className="templates-list-content">
        {isLoading ? (
          <div className="templates-list-loading">
            <Spin />
          </div>
        ) : filteredTemplates.length === 0 ? (
          <Empty
            description={
              filter.searchQuery || filter.favoriteOnly
                ? 'No templates found'
                : 'No templates yet. Create your first template!'
            }
          />
        ) : (
          <Space direction="vertical" size={12} style={{ width: '100%' }}>
            {filteredTemplates.map((template) => (
              <Card
                key={template.id}
                className={`template-card ${
                  template.id === selectedTemplateId ? 'selected' : ''
                }`}
                hoverable
                onClick={() => selectTemplate(template.id)}
              >
                <div className="template-card-header">
                  <Title heading={6} ellipsis={{ rows: 1 }}>
                    {template.title}
                  </Title>
                  <Space size={4}>
                    <Button
                      size="small"
                      type="text"
                      icon={template.isFavorite ? <IconStarFill /> : <IconStar />}
                      onClick={(e) => handleToggleFavorite(template.id, e)}
                    />
                    {!template.isDefault && (
                      <Button
                        size="small"
                        type="text"
                        icon={<IconCheck />}
                        onClick={(e) => handleSetDefault(template.id, e)}
                      />
                    )}
                    <Button
                      size="small"
                      type="text"
                      icon={<IconEdit />}
                      onClick={(e) => {
                        e.stopPropagation();
                        selectTemplate(template.id);
                      }}
                    />
                    <Popconfirm
                      title="Are you sure you want to delete this template?"
                      onOk={(e) => handleDelete(template.id, e as React.MouseEvent<Element, MouseEvent>)}
                      okText="Delete"
                      cancelText="Cancel"
                    >
                      <Button
                        size="small"
                        type="text"
                        status="danger"
                        icon={<IconDelete />}
                        onClick={(e) => e.stopPropagation()}
                      />
                    </Popconfirm>
                  </Space>
                </div>
                <Text
                  ellipsis={{ rows: 2, expandable: false }}
                  style={{ color: 'var(--color-text-2)' }}
                >
                  {template.content}
                </Text>
                <div className="template-card-footer">
                  <Space size={8}>
                    {template.isDefault && <Tag color="arcoblue">Default</Tag>}
                    {template.isFavorite && <Tag color="orangered">Favorite</Tag>}
                  </Space>
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    {new Date(template.updatedAt).toLocaleDateString()}
                  </Text>
                </div>
              </Card>
            ))}
          </Space>
        )}
      </div>
    </div>
  );
}
