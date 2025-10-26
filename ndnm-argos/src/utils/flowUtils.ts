import { type Node, type Edge } from '@xyflow/react';

export type IOmode = 0 | 1 | 'n' | string;

export function isDynInputsNode(n: Node): boolean {
  const m: IOmode | undefined = (n.data as any)?.inputsMode;
  return m === 'n';
}

export function isDynOutputsNode(n: Node): boolean {
  const m: IOmode | undefined = (n.data as any)?.outputsMode;
  return m === 'n';
}

export function normalizeIOMode(mode: string | number | undefined): IOmode {
    if (mode === '0' || mode === 0) return 0;
    if (mode === '1' || mode === 1) return 1;
    if (mode === 'n') return 'n';
    return 1;
}

export function normalizeNodeIOCounts(node: Node, edges: Edge[]): Node {
  let updatedNode = node;
  const data: any = updatedNode.data ?? {};

  const inputsMode = normalizeIOMode(data.inputsMode);
  if (inputsMode === 'n') {
    const currentInCount: number = Number.isFinite(data.inputsCount) ? Math.max(data.inputsCount, 1) : 1;
    const usedInHandles = new Set(
      edges
        .filter((e) => e.target === updatedNode.id && e.targetHandle && String(e.targetHandle).startsWith('in_'))
        .map((e) => String(e.targetHandle))
    );
    const shouldInCount = Math.max(usedInHandles.size + 1, 1);
    if (shouldInCount !== currentInCount) {
      updatedNode = {
          ...updatedNode,
          data: { ...data, inputsMode: 'n', inputsCount: shouldInCount }
      };
    }
  }

  const outputsMode = normalizeIOMode(data.outputsMode);
  if (outputsMode === 'n') {
    const currentData = updatedNode.data ?? {};
    const currentOutCount: number = Number.isFinite(currentData.outputsCount) ? Math.max(currentData.outputsCount, 1) : 1;
    const usedOutHandles = new Set(
      edges
        .filter((e) => e.source === updatedNode.id && e.sourceHandle && String(e.sourceHandle).startsWith('out_'))
        .map((e) => String(e.sourceHandle))
    );
    const shouldOutCount = Math.max(usedOutHandles.size + 1, 1);
    if (shouldOutCount !== currentOutCount) {
      updatedNode = {
          ...updatedNode,
          data: { ...(updatedNode.data ?? {}), outputsMode: 'n', outputsCount: shouldOutCount }
      };
    }
  }

  return updatedNode;
}

export function normalizeAllNodesIO(nodes: Node[], edges: Edge[]): Node[] {
    return nodes.map(n => normalizeNodeIOCounts(n, edges));
}