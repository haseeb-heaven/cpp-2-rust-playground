import React from "react";
import { X } from "lucide-react";

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const SHORTCUTS_LIST = [
  { keys: "Ctrl / Cmd + Enter", description: "Run Rust code" },
  { keys: "Ctrl / Cmd + Shift + Enter", description: "Run C/C++ code" },
  { keys: "Ctrl / Cmd + B", description: "Compile Rust code" },
  { keys: "Ctrl / Cmd + S", description: "Save workspace to local storage" },
  { keys: "Ctrl / Cmd + \\", description: "Swap left and right editor panels" },
  { keys: "Ctrl / Cmd + D", description: "Toggle Diff View" }
];

export const ShortcutsModal: React.FC<ShortcutsModalProps> = ({ isOpen, onClose }) => {
  if (!isOpen) {
    return null;
  }

  return (
    <div className="modal-backdrop" onClick={onClose} data-testid="shortcuts-modal">
      <div className="modal-card modal-small" onClick={(event) => event.stopPropagation()}>
        <div className="modal-header">
          <h2>Keyboard Shortcuts</h2>
          <button type="button" className="icon-button" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </div>
        <ul className="shortcuts-list">
          {SHORTCUTS_LIST.map((shortcut) => (
            <li key={shortcut.keys} className="shortcut-row">
              <kbd className="shortcut-kbd">{shortcut.keys}</kbd>
              <span>{shortcut.description}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
};
