// src/nodes/registry.ts
// Define apenas a interface para os dados que vêm do backend.

export interface NodePaletteItem {
  type: string;
  label: string;
  default_data: any;
}