import React, { useState } from "react";
import { X } from "lucide-react";
import type { CodeExample } from "../types/studio";

interface ExamplesModalProps {
  isOpen: boolean;
  examples: CodeExample[];
  onSelectExample: (example: CodeExample) => void;
  onClose: () => void;
}

const DIFFICULTY_FILTERS = ["All", "Easy", "Medium", "Hard", "Very Complex"] as const;

export const ExamplesModal: React.FC<ExamplesModalProps> = ({
  isOpen,
  examples,
  onSelectExample,
  onClose
}) => {
  const [selectedDifficulty, setSelectedDifficulty] = useState<string>("All");

  if (!isOpen) {
    return null;
  }

  const visibleExamples =
    selectedDifficulty === "All"
      ? examples
      : examples.filter((example) => example.difficulty === selectedDifficulty);

  return (
    <div className="modal-backdrop" onClick={onClose} data-testid="examples-modal">
      <div className="modal-card" onClick={(event) => event.stopPropagation()}>
        <div className="modal-header">
          <h2>Pre-made Examples ({visibleExamples.length})</h2>
          <button type="button" className="icon-button" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </div>

        <fieldset className="difficulty-filter-bar" aria-label="Filter by difficulty">
          {DIFFICULTY_FILTERS.map((difficulty) => (
            <button
              key={difficulty}
              type="button"
              className={`action-button ${selectedDifficulty === difficulty ? "active" : ""}`}
              onClick={() => setSelectedDifficulty(difficulty)}
            >
              {difficulty}
            </button>
          ))}
        </fieldset>

        <div className="examples-grid">
          {visibleExamples.map((example) => (
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
                <span className="example-badge">{example.difficulty}</span>
              </div>
              <div className="example-category-line">{example.category}</div>
              <p className="example-description">{example.description}</p>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
