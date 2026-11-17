import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { ChevronDown, Download, Save } from "lucide-react";

export function SaveMenu({ sourceLabel, onSaveWorkspace, onExportSource, onExportRust, statusMessage }: {
  sourceLabel: string;
  onSaveWorkspace: () => void;
  onExportSource: () => void;
  onExportRust: () => void;
  statusMessage: string | null;
}) {
  return <DropdownMenu.Root>
    <DropdownMenu.Trigger className="action-button shadcn-save-trigger" data-testid="save-menu-button">
      <Save size={15} /><span>{statusMessage ?? "Save"}</span><ChevronDown size={14} />
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal><DropdownMenu.Content className="shadcn-menu-content" align="end">
      <DropdownMenu.Item className="shadcn-menu-item" onSelect={onSaveWorkspace}><Save size={14} /> Save workspace</DropdownMenu.Item>
      <DropdownMenu.Separator className="shadcn-menu-separator" />
      <DropdownMenu.Item className="shadcn-menu-item" onSelect={onExportSource}><Download size={14} /> Export .{sourceLabel}</DropdownMenu.Item>
      <DropdownMenu.Item className="shadcn-menu-item" onSelect={onExportRust}><Download size={14} /> Export .rs</DropdownMenu.Item>
    </DropdownMenu.Content></DropdownMenu.Portal>
  </DropdownMenu.Root>;
}
