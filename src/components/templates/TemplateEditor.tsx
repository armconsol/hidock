import { useState, useEffect } from 'react';
import {
  Space,
  Input,
  Button,
  Checkbox,
  message,
  Typography,
  Card,
} from 'antd';
import { SaveOutlined, CloseOutlined } from '@ant-design/icons';
import { useTemplatesStore } from '../../store/templatesStore';
import './TemplateEditor.css';

const { TextArea } = Input;
const { Title } = Typography;

interface TemplateEditorProps {
  templateId: string | null;
  onClose?: () => void;
}

export function TemplateEditor({ templateId, onClose }: TemplateEditorProps) {
  const { getTemplate, createTemplate, updateTemplate } = useTemplatesStore();

  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [isFavorite, setIsFavorite] = useState(false);
  const [isDefault, setIsDefault] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const isNewTemplate = templateId === 'new';

  useEffect(() => {
    if (templateId && templateId !== 'new') {
      loadTemplate(templateId);
    } else {
      // Reset form for new template
      setTitle('');
      setContent('');
      setIsFavorite(false);
      setIsDefault(false);
    }
  }, [templateId]);

  const loadTemplate = async (id: string) => {
    setIsLoading(true);
    try {
      const template = await getTemplate(id);
      if (template) {
        setTitle(template.title);
        setContent(template.content);
        setIsFavorite(template.isFavorite);
        setIsDefault(template.isDefault);
      }
    } catch (error) {
      message.error('Failed to load template');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    if (!title.trim()) {
      message.warning('Please enter a title');
      return;
    }

    if (!content.trim()) {
      message.warning('Please enter content');
      return;
    }

    setIsSaving(true);
    try {
      if (isNewTemplate) {
        await createTemplate(title, content, isFavorite, isDefault);
        message.success('Template created successfully');
      } else if (templateId) {
        await updateTemplate(templateId, {
          title,
          content,
          isFavorite,
          isDefault,
        });
        message.success('Template updated successfully');
      }
      onClose?.();
    } catch (error) {
      message.error(
        isNewTemplate ? 'Failed to create template' : 'Failed to update template'
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleCancel = () => {
    setTitle('');
    setContent('');
    setIsFavorite(false);
    setIsDefault(false);
    onClose?.();
  };

  if (!templateId) {
    return (
      <div className="template-editor-empty">
        <Typography.Text type="secondary">
          Select a template to edit or create a new one
        </Typography.Text>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="template-editor-loading">
        <Typography.Text>Loading template...</Typography.Text>
      </div>
    );
  }

  return (
    <div className="template-editor">
      <Card className="template-editor-card">
        <div className="template-editor-header">
          <Title level={5}>{isNewTemplate ? 'New Template' : 'Edit Template'}</Title>
          <Space size={8}>
            <Button icon={<CloseOutlined />} onClick={handleCancel}>
              Cancel
            </Button>
            <Button
              type="primary"
              icon={<SaveOutlined />}
              loading={isSaving}
              onClick={handleSave}
            >
              Save
            </Button>
          </Space>
        </div>

        <Space direction="vertical" size={16} style={{ width: '100%' }}>
          <div>
            <Typography.Text>Title</Typography.Text>
            <Input
              placeholder="Template title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              style={{ marginTop: 8 }}
            />
          </div>

          <div>
            <Typography.Text>Content</Typography.Text>
            <TextArea
              placeholder="Template content"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              style={{ marginTop: 8, minHeight: 300 }}
              autoSize={{ minRows: 10, maxRows: 20 }}
            />
          </div>

          <Space direction="vertical" size={8}>
            <Checkbox checked={isFavorite} onChange={(e) => setIsFavorite(e.target.checked)}>
              Mark as favorite
            </Checkbox>
            <Checkbox checked={isDefault} onChange={(e) => setIsDefault(e.target.checked)}>
              Set as default template
            </Checkbox>
          </Space>
        </Space>
      </Card>
    </div>
  );
}
