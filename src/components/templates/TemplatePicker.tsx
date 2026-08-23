import { useState, useEffect } from 'react';
import {
  Modal,
  Space,
  Input,
  Button,
  Empty,
  List,
  Typography,
  Tag,
  Radio,
} from 'antd';
import { SearchOutlined, StarOutlined, StarFilled } from '@ant-design/icons';
import { useTemplatesStore } from '../../store/templatesStore';
import type { Template } from '../../types/templates';
import './TemplatePicker.css';

const { Text } = Typography;

interface TemplatePickerProps {
  visible: boolean;
  onClose: () => void;
  onSelect: (template: Template) => void;
  autoSelectDefault?: boolean;
}

export function TemplatePicker({
  visible,
  onClose,
  onSelect,
  autoSelectDefault = false,
}: TemplatePickerProps) {
  const { templates, loadTemplates, getDefaultTemplate } = useTemplatesStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null);
  const [filterFavorite, setFilterFavorite] = useState(false);

  useEffect(() => {
    if (visible) {
      loadTemplates();

      // Auto-select default template if enabled
      if (autoSelectDefault) {
        loadDefaultTemplate();
      }
    }
  }, [visible, autoSelectDefault]);

  const loadDefaultTemplate = async () => {
    try {
      const defaultTemplate = await getDefaultTemplate();
      if (defaultTemplate) {
        setSelectedTemplateId(defaultTemplate.id);
      }
    } catch (error) {
      console.error('Failed to load default template:', error);
    }
  };

  const filteredTemplates = templates.filter((template) => {
    // Filter by search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      if (
        !template.title.toLowerCase().includes(query) &&
        !template.content.toLowerCase().includes(query)
      ) {
        return false;
      }
    }

    // Filter by favorite
    if (filterFavorite && !template.isFavorite) {
      return false;
    }

    return true;
  });

  const handleSelect = () => {
    const template = templates.find((t) => t.id === selectedTemplateId);
    if (template) {
      onSelect(template);
      handleClose();
    }
  };

  const handleClose = () => {
    setSearchQuery('');
    setSelectedTemplateId(null);
    setFilterFavorite(false);
    onClose();
  };

  return (
    <Modal
      title="Select Template"
      open={visible}
      onCancel={handleClose}
      onOk={handleSelect}
      okText="Use Template"
      okButtonProps={{ disabled: !selectedTemplateId }}
      width={600}
    >
      <Space direction="vertical" size={12} style={{ width: '100%' }}>
        <Space size={8} style={{ width: '100%' }}>
          <Input
            allowClear
            placeholder="Search templates..."
            prefix={<SearchOutlined />}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{ flex: 1 }}
          />
          <Button
            icon={filterFavorite ? <StarFilled /> : <StarOutlined />}
            type={filterFavorite ? 'primary' : 'default'}
            onClick={() => setFilterFavorite(!filterFavorite)}
          />
        </Space>

        <div className="template-picker-list">
          {filteredTemplates.length === 0 ? (
            <Empty
              description={
                searchQuery || filterFavorite
                  ? 'No templates found'
                  : 'No templates available'
              }
            />
          ) : (
            <Radio.Group
              value={selectedTemplateId}
              onChange={(e) => setSelectedTemplateId(e.target.value)}
              style={{ width: '100%' }}
            >
              <List
                dataSource={filteredTemplates}
                renderItem={(template) => (
                  <List.Item key={template.id} className="template-picker-item">
                    <Radio value={template.id}>
                      <div className="template-picker-item-content">
                        <div className="template-picker-item-header">
                          <Text style={{ fontWeight: 500 }}>{template.title}</Text>
                          <Space size={4}>
                            {template.isDefault && <Tag color="blue">Default</Tag>}
                            {template.isFavorite && <Tag color="orange">Favorite</Tag>}
                          </Space>
                        </div>
                        <Text
                          type="secondary"
                          ellipsis={{ tooltip: template.content }}
                          style={{ fontSize: 12, marginTop: 4 }}
                        >
                          {template.content}
                        </Text>
                      </div>
                    </Radio>
                  </List.Item>
                )}
              />
            </Radio.Group>
          )}
        </div>
      </Space>
    </Modal>
  );
}
