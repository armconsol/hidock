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
} from '@arco-design/web-react';
import { IconSearch, IconStar, IconStarFill } from '@arco-design/web-react/icon';
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
      visible={visible}
      onCancel={handleClose}
      onOk={handleSelect}
      okText="Use Template"
      okButtonProps={{ disabled: !selectedTemplateId }}
      style={{ width: 600 }}
    >
      <Space direction="vertical" size={12} style={{ width: '100%' }}>
        <Space size={8} style={{ width: '100%' }}>
          <Input
            allowClear
            placeholder="Search templates..."
            prefix={<IconSearch />}
            value={searchQuery}
            onChange={setSearchQuery}
            style={{ flex: 1 }}
          />
          <Button
            icon={filterFavorite ? <IconStarFill /> : <IconStar />}
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
              onChange={setSelectedTemplateId}
              style={{ width: '100%' }}
            >
              <List
                dataSource={filteredTemplates}
                render={(template) => (
                  <List.Item key={template.id} className="template-picker-item">
                    <Radio value={template.id}>
                      <div className="template-picker-item-content">
                        <div className="template-picker-item-header">
                          <Text style={{ fontWeight: 500 }}>{template.title}</Text>
                          <Space size={4}>
                            {template.isDefault && <Tag color="arcoblue">Default</Tag>}
                            {template.isFavorite && <Tag color="orangered">Favorite</Tag>}
                          </Space>
                        </div>
                        <Text
                          type="secondary"
                          ellipsis={{ rows: 2, expandable: false }}
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
