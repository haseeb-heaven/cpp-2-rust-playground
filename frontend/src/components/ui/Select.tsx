import * as SelectPrimitive from "@radix-ui/react-select";
import { Check, ChevronDown } from "lucide-react";
import type { ReactNode } from "react";

export function Select({ value, onValueChange, ariaLabel, children, testId }: {
  value: string;
  onValueChange: (value: string) => void;
  ariaLabel: string;
  children: ReactNode;
  testId?: string;
}) {
  return <SelectPrimitive.Root value={value} onValueChange={onValueChange}>
    <SelectPrimitive.Trigger className="shadcn-select-trigger" aria-label={ariaLabel} data-testid={testId}>
      <SelectPrimitive.Value />
      <SelectPrimitive.Icon><ChevronDown size={14} /></SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
    <SelectPrimitive.Portal><SelectPrimitive.Content className="shadcn-select-content" position="popper">
      <SelectPrimitive.Viewport>{children}</SelectPrimitive.Viewport>
    </SelectPrimitive.Content></SelectPrimitive.Portal>
  </SelectPrimitive.Root>;
}

export function SelectItem({ value, children }: { value: string; children: ReactNode }) {
  return <SelectPrimitive.Item value={value} className="shadcn-select-item">
    <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
    <SelectPrimitive.ItemIndicator><Check size={14} /></SelectPrimitive.ItemIndicator>
  </SelectPrimitive.Item>;
}
