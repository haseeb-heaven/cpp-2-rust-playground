import React from "react";
import { X } from "lucide-react";
import type { CodeExample } from "../types/studio";

interface ExamplesModalProps {
  isOpen: boolean;
  examples: CodeExample[];
  onSelectExample: (example: CodeExample) => void;
  onClose: () => void;
}

export const ExamplesModal: React.FC<ExamplesModalProps> = ({
  isOpen,
  examples,
  onSelectExample,
  onClose
}) => {
  if (!isOpen) {
    return null;
  }

  return (
    <div className="modal-backdrop" onClick={onClose} data-testid="examples-modal">
      <div className="modal-card" onClick={(event) => event.stopPropagation()}>
        <div className="modal-header">
          <h2>Pre-made Examples</h2>
          <button type="button" className="icon-button" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </div>
        <div className="examples-grid">
          {examples.map((example) => (
            <button
              key={example.identifier}
              type="button"
              className="example-card"
              onClick={() => {
                onSelectExample(example);
                onClose();
              }}
              data-testid={`example-${example.identifier}`}
            >
              <div className="example-card-top">
                <span className="example-title">{example.title}</span>
                <span className="example-badge">{example.category}</span>
              </div>
              <p className="example-description">{example.description}</p>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
